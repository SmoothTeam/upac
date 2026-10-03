// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use redb::{ReadableDatabase, ReadableTable};

use twox_hash::xxhash3_64::Hasher as XxHasher;

use uuid::Uuid;

use upac_types::response::entry::FileEntry;

use super::error::DatabaseError;
use super::{
    FILES_UUID_HASH_TABLE, FILES_UUID_TABLE, MemoryDatabase, ReadTransactionExt, ReadableSource, record_decode,
    record_encode,
};

pub trait FileStore {
    fn path_hash(path: &str) -> u64 {
        XxHasher::oneshot(path.as_bytes())
    }

    fn find_file_owner(&self, path: &str) -> Result<Option<Uuid>, DatabaseError>;
    fn list_package_files(&self, uuid: Uuid) -> Result<Vec<FileEntry>, DatabaseError>;
    fn list_files(&self) -> Result<Vec<(Uuid, FileEntry)>, DatabaseError>;
}

pub trait FileStoreMut: FileStore {
    fn insert_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError>;
    fn update_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError>;
    fn remove_package_file(&mut self, uuid: Uuid, path: &str) -> Result<FileEntry, DatabaseError>;
    fn remove_user_file(&mut self, uuid: Uuid, path: &str) -> Result<FileEntry, DatabaseError>;
}

impl<T: ReadableSource> FileStore for T {
    fn find_file_owner(&self, path: &str) -> Result<Option<Uuid>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(by_path) = transaction.open_table_or_none(FILES_UUID_HASH_TABLE)? else {
            return Ok(None);
        };

        Ok(by_path.get(Self::path_hash(path))?.map(|guard| guard.value()))
    }

    fn list_package_files(&self, uuid: Uuid) -> Result<Vec<FileEntry>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(files) = transaction.open_table_or_none(FILES_UUID_TABLE)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();

        for entry in files.range((uuid, 0u64)..)? {
            let (key, value) = entry?;
            let (row_uuid, _hash) = key.value();

            if row_uuid != uuid {
                break;
            }

            out.push(record_decode(value.value())?);
        }

        Ok(out)
    }

    fn list_files(&self) -> Result<Vec<(Uuid, FileEntry)>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(files) = transaction.open_table_or_none(FILES_UUID_TABLE)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();

        for entry in files.iter()? {
            let (key, value) = entry?;
            let (uuid, _hash) = key.value();

            out.push((uuid, record_decode(value.value())?));
        }

        Ok(out)
    }
}

impl FileStoreMut for MemoryDatabase {
    fn insert_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError> {
        let hash = Self::path_hash(&entry.path);
        let transaction = self.database.begin_write()?;
        let mut files = transaction.open_table(FILES_UUID_TABLE)?;

        let already_user_owned = match files.get((uuid, hash))? {
            Some(existing) => record_decode::<FileEntry>(existing.value())?.is_user,
            None => false,
        };

        if already_user_owned {
            return Ok(());
        }

        files.insert((uuid, hash), record_encode(entry)?.as_slice())?;

        drop(files);
        transaction.open_table(FILES_UUID_HASH_TABLE)?.insert(hash, uuid)?;
        transaction.commit()?;

        Ok(())
    }

    fn update_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError> {
        self.insert_package_file(uuid, entry)
    }

    fn remove_package_file(&mut self, uuid: Uuid, path: &str) -> Result<FileEntry, DatabaseError> {
        let hash = Self::path_hash(path);
        let transaction = self.database.begin_write()?;
        let mut files = transaction.open_table(FILES_UUID_TABLE)?;

        let entry: FileEntry = record_decode(files.get((uuid, hash))?.ok_or(DatabaseError::FileNotFound)?.value())?;

        if entry.is_user {
            return Err(DatabaseError::AccessDenied);
        }

        files.remove((uuid, hash))?;

        drop(files);
        transaction.open_table(FILES_UUID_HASH_TABLE)?.remove(hash)?;
        transaction.commit()?;

        Ok(entry)
    }

    fn remove_user_file(&mut self, uuid: Uuid, path: &str) -> Result<FileEntry, DatabaseError> {
        let hash = Self::path_hash(path);
        let transaction = self.database.begin_write()?;
        let mut files = transaction.open_table(FILES_UUID_TABLE)?;

        let entry: FileEntry = record_decode(files.get((uuid, hash))?.ok_or(DatabaseError::FileNotFound)?.value())?;

        if !entry.is_user {
            return Err(DatabaseError::AccessDenied);
        }

        files.remove((uuid, hash))?;

        drop(files);
        transaction.open_table(FILES_UUID_HASH_TABLE)?.remove(hash)?;
        transaction.commit()?;

        Ok(entry)
    }
}
