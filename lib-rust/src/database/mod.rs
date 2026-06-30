use std::path::Path;

use libc::{ENOENT, EPERM, EROFS};
use libmdbx::{
    Database as Env, DatabaseOptions, Error as MdbxError, Mode, NoWriteMap, RO, RW, ReadWriteOptions, TableFlags,
    Transaction, TransactionKind, WriteFlags,
};
use uuid::Uuid;

use crate::types::errors::DatabaseError;

pub mod files;
pub mod packages;

const PACKAGES_TABLE: &str = "packages";
const FILES_TABLE: &str = "files";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Store {
    Packages,
    Files,
}

impl Store {
    fn name(self) -> &'static str {
        match self {
            Store::Packages => PACKAGES_TABLE,
            Store::Files => FILES_TABLE,
        }
    }
}

impl From<MdbxError> for DatabaseError {
    fn from(error: MdbxError) -> Self {
        match error {
            MdbxError::Access | MdbxError::Other(EPERM) | MdbxError::Other(EROFS) => DatabaseError::AccessDenied,

            MdbxError::NotFound | MdbxError::Other(ENOENT) => DatabaseError::DatabaseNotInitialized,

            MdbxError::KeyExist => DatabaseError::PackageAlreadyExists,

            MdbxError::MapFull
            | MdbxError::TxnFull
            | MdbxError::DbsFull
            | MdbxError::CursorFull
            | MdbxError::PageFull
            | MdbxError::UnableExtendMapsize => DatabaseError::WriteError,

            _ => DatabaseError::ReadError,
        }
    }
}

pub struct Database {
    env: Env<NoWriteMap>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, DatabaseError> {
        let env = Self::open_env(path, Mode::ReadWrite(ReadWriteOptions::default()))?;

        let txn = env.begin_rw_txn()?;

        txn.create_table(Some(PACKAGES_TABLE), TableFlags::empty())?;
        txn.create_table(Some(FILES_TABLE), TableFlags::DUP_SORT)?;

        txn.commit()?;

        Ok(Self { env })
    }

    pub fn open_rw(path: &Path) -> Result<Self, DatabaseError> {
        let env = Self::open_env(path, Mode::ReadWrite(ReadWriteOptions::default()))?;

        Ok(Self { env })
    }

    pub fn open_ro(path: &Path) -> Result<Self, DatabaseError> {
        let env = Self::open_env(path, Mode::ReadOnly)?;

        Ok(Self { env })
    }

    fn open_env(path: &Path, mode: Mode) -> Result<Env<NoWriteMap>, DatabaseError> {
        let database_options = DatabaseOptions {
            max_tables: Some(2),
            no_sub_dir: true,
            mode,
            ..Default::default()
        };

        Ok(Env::open_with_options(path, database_options)?)
    }
}

pub struct DatabaseTransaction<'db, K: TransactionKind> {
    transaction: Transaction<'db, K, NoWriteMap>,
}

impl<'db> DatabaseTransaction<'db, RW> {
    pub fn begin_rw(database: &'db Database) -> Result<Self, DatabaseError> {
        Ok(Self {
            transaction: database.env.begin_rw_txn()?,
        })
    }

    pub(crate) fn put(&self, store: Store, key: Uuid, value: &[u8]) -> Result<(), DatabaseError> {
        let table = self.transaction.open_table(Some(store.name()))?;

        self.transaction
            .put(&table, key.as_bytes(), value, WriteFlags::UPSERT)?;

        Ok(())
    }

    pub(crate) fn delete(&self, store: Store, key: Uuid) -> Result<(), DatabaseError> {
        let table = self.transaction.open_table(Some(store.name()))?;

        self.transaction.del(&table, key.as_bytes(), None)?;

        Ok(())
    }

    pub(crate) fn delete_value(&self, store: Store, key: Uuid, value: &[u8]) -> Result<(), DatabaseError> {
        let table = self.transaction.open_table(Some(store.name()))?;

        self.transaction.del(&table, key.as_bytes(), Some(value))?;

        Ok(())
    }
}

impl<'db> DatabaseTransaction<'db, RO> {
    pub fn begin_ro(database: &'db Database) -> Result<Self, DatabaseError> {
        Ok(Self {
            transaction: database.env.begin_ro_txn()?,
        })
    }
}

impl<'db, K: TransactionKind> DatabaseTransaction<'db, K> {
    pub(crate) fn entries(&self, store: Store) -> Result<Vec<(Uuid, Vec<u8>)>, DatabaseError> {
        let table = self.transaction.open_table(Some(store.name()))?;
        let mut cursor = self.transaction.cursor(&table)?;

        let mut out = Vec::new();
        let mut entry = cursor.first::<Vec<u8>, Vec<u8>>()?;

        while let Some((key, value)) = entry {
            let uuid = Uuid::from_slice(&key).map_err(|_| DatabaseError::ReadError)?;
            out.push((uuid, value));
            entry = cursor.next::<Vec<u8>, Vec<u8>>()?;
        }

        Ok(out)
    }

    pub(crate) fn values_of(&self, store: Store, key: Uuid) -> Result<Vec<Vec<u8>>, DatabaseError> {
        let table = self.transaction.open_table(Some(store.name()))?;
        let mut cursor = self.transaction.cursor(&table)?;

        let mut out = Vec::new();
        let positioned = cursor.set_lowerbound::<Vec<u8>, Vec<u8>>(key.as_bytes(), None)?;

        let mut current = match positioned {
            Some((_next, found, value)) if found.as_slice() == key.as_bytes() => Some(value),
            _ => return Ok(out),
        };

        while let Some(value) = current {
            out.push(value);
            current = cursor.next_dup::<Vec<u8>, Vec<u8>>()?.map(|(_k, v)| v);
        }

        Ok(out)
    }

    pub fn commit(self) -> Result<(), DatabaseError> {
        self.transaction.commit()?;
        Ok(())
    }

    pub fn abort(self) {
        drop(self);
    }
}
