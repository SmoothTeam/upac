// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use redb::{ReadableDatabase, ReadableTable, TypeName, Value as RedbValue};
use twox_hash::xxhash3_64::Hasher as XxHasher;
use uuid::Uuid;

use super::error::DatabaseError;
use super::{FILES_UUID_HASH_TABLE, FILES_UUID_TABLE, MemoryDatabase, ReadableSource};

use crate::types::FileEntry;
use crate::types::database::FILES_ENTRY_TYPE_NAME;

pub trait FileStore {
    fn path_hash(path: &str) -> u64 {
        XxHasher::oneshot(path.as_bytes())
    }

    fn find_file_owner(&self, path: &str) -> Result<Option<Uuid>, DatabaseError>;
    fn list_files(&self, uuid: Uuid) -> Result<Vec<FileEntry>, DatabaseError>;
}

pub trait FileStoreMut: FileStore {
    fn insert_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError>;
    fn update_package_file(&mut self, uuid: Uuid, entry: &FileEntry) -> Result<(), DatabaseError>;
    fn remove_package_file(&mut self, uuid: Uuid, path: &str) -> Result<FileEntry, DatabaseError>;
}

impl<T: ReadableSource> FileStore for T {
    fn find_file_owner(&self, path: &str) -> Result<Option<Uuid>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let by_path = transaction.open_table(FILES_UUID_HASH_TABLE)?;

        Ok(by_path.get(Self::path_hash(path))?.map(|guard| guard.value()))
    }

    fn list_files(&self, uuid: Uuid) -> Result<Vec<FileEntry>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let files = transaction.open_table(FILES_UUID_TABLE)?;
        let mut out = Vec::new();

        for entry in files.range((uuid, 0u64)..)? {
            let (key, value) = entry?;
            let (row_uuid, _hash) = key.value();

            if row_uuid != uuid {
                break;
            }

            out.push(value.value());
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
            Some(existing) => existing.value().is_user,
            None => false,
        };

        if already_user_owned {
            return Ok(());
        }

        files.insert((uuid, hash), entry)?;

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

        let entry = files.get((uuid, hash))?.ok_or(DatabaseError::FileNotFound)?.value();

        if entry.is_user {
            return Err(DatabaseError::AccessDenied);
        }

        files.remove((uuid, hash))?;

        drop(files);
        transaction.open_table(FILES_UUID_HASH_TABLE)?.remove(hash)?;
        transaction.commit()?;

        Ok(entry)
    }
}

impl RedbValue for FileEntry {
    type AsBytes<'a> = Vec<u8>;
    type SelfType<'a> = FileEntry;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> FileEntry
    where
        Self: 'a,
    {
        let mut offset = 0;

        FileEntry::decode_from(data, &mut offset)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a FileEntry) -> Vec<u8>
    where
        Self: 'b,
    {
        let mut buf = Vec::new();

        FileEntry::encode_into(&mut buf, value);
        buf
    }

    fn type_name() -> TypeName {
        TypeName::new(FILES_ENTRY_TYPE_NAME)
    }
}
