// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Error as IoError, ErrorKind};
use std::sync::{Arc, PoisonError, RwLock};

use redb::{
    Builder, Database as RedbDatabase, Key, ReadOnlyTable, ReadTransaction, StorageBackend, TableDefinition,
    TableError, Value,
};

use serde::Serialize;
use serde::de::DeserializeOwned;

use uuid::Uuid;

use self::error::DatabaseError;
use self::layout::database::{
    FILES_BY_PATH_TABLE_NAME, FILES_TABLE_NAME, PACKAGES_BY_NAME_TABLE_NAME, PACKAGES_TABLE_NAME,
    PACKAGES_TRIGGERS_TABLE_NAME, TRANSACTION_TABLE_NAME,
};

pub mod attribution;
pub mod error;
pub mod files;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod meta;
pub mod transaction;
pub mod triggers;

pub(crate) const PACKAGES_UUID_TABLE: TableDefinition<Uuid, &[u8]> = TableDefinition::new(PACKAGES_TABLE_NAME);
pub(crate) const PACKAGES_HASH_TABLE: TableDefinition<u64, Uuid> = TableDefinition::new(PACKAGES_BY_NAME_TABLE_NAME);
pub(crate) const PACKAGES_TRIGGERS_TABLE: TableDefinition<Uuid, &[u8]> =
    TableDefinition::new(PACKAGES_TRIGGERS_TABLE_NAME);

pub(crate) const FILES_UUID_TABLE: TableDefinition<(Uuid, u64), &[u8]> = TableDefinition::new(FILES_TABLE_NAME);
pub(crate) const FILES_UUID_HASH_TABLE: TableDefinition<u64, Uuid> = TableDefinition::new(FILES_BY_PATH_TABLE_NAME);

pub(crate) const TRANSACTION_TABLE: TableDefinition<(), &[u8]> = TableDefinition::new(TRANSACTION_TABLE_NAME);

pub(crate) fn record_encode<Record: Serialize + ?Sized>(record: &Record) -> Result<Vec<u8>, DatabaseError> {
    postcard::to_allocvec(record).map_err(|_| DatabaseError::WriteError)
}

pub(crate) fn record_decode<Record: DeserializeOwned>(bytes: &[u8]) -> Result<Record, DatabaseError> {
    postcard::from_bytes(bytes).map_err(|_| DatabaseError::ReadError)
}

pub struct MemoryDatabase {
    database: RedbDatabase,
    backend: SharedMemoryBackend,
}

impl MemoryDatabase {
    pub fn new_in_memory() -> Result<Self, DatabaseError> {
        let backend = SharedMemoryBackend::new();
        let database = Builder::new().create_with_backend(backend.clone())?;

        Ok(Self { database, backend })
    }

    pub fn open_in_memory(bytes: Vec<u8>) -> Result<Self, DatabaseError> {
        let backend = SharedMemoryBackend(Arc::new(RwLock::new(bytes)));
        let database = Builder::new().create_with_backend(backend.clone())?;

        Ok(Self { database, backend })
    }

    pub fn into_bytes(self) -> Result<Vec<u8>, DatabaseError> {
        drop(self.database);

        Ok(self.backend.into_bytes())
    }
}

pub(crate) trait ReadTransactionExt {
    fn open_table_or_none<TableKey: Key + 'static, TableValue: Value + 'static>(
        &self, definition: TableDefinition<TableKey, TableValue>,
    ) -> Result<Option<ReadOnlyTable<TableKey, TableValue>>, DatabaseError>;
}

impl ReadTransactionExt for ReadTransaction {
    fn open_table_or_none<TableKey: Key + 'static, TableValue: Value + 'static>(
        &self, definition: TableDefinition<TableKey, TableValue>,
    ) -> Result<Option<ReadOnlyTable<TableKey, TableValue>>, DatabaseError> {
        match self.open_table(definition) {
            Ok(table) => Ok(Some(table)),
            Err(TableError::TableDoesNotExist(_)) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SharedMemoryBackend(Arc<RwLock<Vec<u8>>>);

impl SharedMemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match Arc::try_unwrap(self.0) {
            Ok(lock) => lock.into_inner().unwrap_or_else(PoisonError::into_inner),
            Err(shared) => shared.read().unwrap_or_else(PoisonError::into_inner).clone(),
        }
    }
}

impl StorageBackend for SharedMemoryBackend {
    fn len(&self) -> Result<u64, IoError> {
        let buffer = self.0.read().unwrap_or_else(PoisonError::into_inner);

        Ok(buffer.len() as u64)
    }

    fn read(&self, offset: u64, out: &mut [u8]) -> Result<(), IoError> {
        let buffer = self.0.read().unwrap_or_else(PoisonError::into_inner);
        let offset = offset as usize;

        let Some(source) = buffer.get(offset..offset + out.len()) else {
            return Err(IoError::from(ErrorKind::UnexpectedEof));
        };

        out.copy_from_slice(source);
        Ok(())
    }

    fn set_len(&self, len: u64) -> Result<(), IoError> {
        let mut buffer = self.0.write().unwrap_or_else(PoisonError::into_inner);

        buffer.resize(len as usize, 0);
        Ok(())
    }

    fn sync_data(&self) -> Result<(), IoError> {
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), IoError> {
        let mut buffer = self.0.write().unwrap_or_else(PoisonError::into_inner);
        let offset = offset as usize;

        let Some(destination) = buffer.get_mut(offset..offset + data.len()) else {
            return Err(IoError::from(ErrorKind::UnexpectedEof));
        };

        destination.copy_from_slice(data);
        Ok(())
    }
}
