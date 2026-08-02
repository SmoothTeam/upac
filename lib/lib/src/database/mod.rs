// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::io::{Error as IoError, ErrorKind};
use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock};

use redb::{
    Builder, Database as RedbDatabase, ReadOnlyDatabase as RedbReadOnlyDatabase, ReadableDatabase, StorageBackend,
    TableDefinition,
};
use uuid::Uuid;

use crate::types::FileEntry;
use crate::types::PackageMeta;
use crate::types::database::{
    FILES_BY_PATH_TABLE_NAME, FILES_TABLE_NAME, PACKAGES_BY_NAME_TABLE_NAME, PACKAGES_TABLE_NAME,
};

pub use self::error::DatabaseError;

mod error;
pub mod files;
pub mod meta;

pub(crate) const PACKAGES_UUID_TABLE: TableDefinition<Uuid, PackageMeta> = TableDefinition::new(PACKAGES_TABLE_NAME);
pub(crate) const PACKAGES_HASH_TABLE: TableDefinition<u64, Uuid> = TableDefinition::new(PACKAGES_BY_NAME_TABLE_NAME);

pub(crate) const FILES_UUID_TABLE: TableDefinition<(Uuid, u64), FileEntry> = TableDefinition::new(FILES_TABLE_NAME);
pub(crate) const FILES_UUID_HASH_TABLE: TableDefinition<u64, Uuid> = TableDefinition::new(FILES_BY_PATH_TABLE_NAME);

pub trait InMemory {
    fn new_in_memory() -> Result<Self, DatabaseError>
    where
        Self: Sized;

    fn open_in_memory(bytes: Vec<u8>) -> Result<Self, DatabaseError>
    where
        Self: Sized;

    fn into_bytes(self) -> Result<Vec<u8>, DatabaseError>
    where
        Self: Sized;
}

pub trait FromFile {
    fn open_from_file(path: &Path) -> Result<Self, DatabaseError>
    where
        Self: Sized;
}

pub struct MemoryDatabase {
    database: RedbDatabase,
    backend: SharedMemoryBackend,
}

impl InMemory for MemoryDatabase {
    fn new_in_memory() -> Result<Self, DatabaseError> {
        let backend = SharedMemoryBackend::new();
        let database = Builder::new().create_with_backend(backend.clone())?;

        Ok(Self { database, backend })
    }

    fn open_in_memory(bytes: Vec<u8>) -> Result<Self, DatabaseError> {
        let backend = SharedMemoryBackend(Arc::new(RwLock::new(bytes)));
        let database = Builder::new().create_with_backend(backend.clone())?;

        Ok(Self { database, backend })
    }

    fn into_bytes(self) -> Result<Vec<u8>, DatabaseError> {
        drop(self.database);

        Ok(self.backend.into_bytes())
    }
}

pub struct ReadOnlyDatabase {
    database: RedbReadOnlyDatabase,
}

impl FromFile for ReadOnlyDatabase {
    fn open_from_file(path: &Path) -> Result<Self, DatabaseError> {
        let database = RedbReadOnlyDatabase::open(path)?;

        Ok(Self { database })
    }
}

pub(crate) trait ReadableSource {
    type Source: ReadableDatabase;

    fn source(&self) -> &Self::Source;
}

impl ReadableSource for MemoryDatabase {
    type Source = RedbDatabase;

    fn source(&self) -> &RedbDatabase {
        &self.database
    }
}

impl ReadableSource for ReadOnlyDatabase {
    type Source = RedbReadOnlyDatabase;

    fn source(&self) -> &RedbReadOnlyDatabase {
        &self.database
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedMemoryBackend(Arc<RwLock<Vec<u8>>>);

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

pub(crate) mod codec {
    pub(crate) fn write_bool(buf: &mut Vec<u8>, value: bool) {
        buf.push(u8::from(value));
    }

    pub(crate) fn write_u32(buf: &mut Vec<u8>, value: u32) {
        buf.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u64(buf: &mut Vec<u8>, value: u64) {
        buf.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
        write_u32(buf, bytes.len() as u32);
        buf.extend_from_slice(bytes);
    }

    pub(crate) fn write_vec_u32(buf: &mut Vec<u8>, values: &[u32]) {
        write_u32(buf, values.len() as u32);

        for value in values {
            write_u32(buf, *value);
        }
    }

    pub(crate) fn write_opt_str(buf: &mut Vec<u8>, value: Option<&str>) {
        match value {
            Some(text) => {
                buf.push(1);
                write_len_prefixed(buf, text.as_bytes());
            }
            None => buf.push(0),
        }
    }

    pub(crate) fn read_bool(data: &[u8], offset: &mut usize) -> bool {
        let value = data[*offset] != 0;
        *offset += 1;

        value
    }

    pub(crate) fn read_u32(data: &[u8], offset: &mut usize) -> u32 {
        let value = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        *offset += 4;

        value
    }

    pub(crate) fn read_u64(data: &[u8], offset: &mut usize) -> u64 {
        let value = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
        *offset += 8;

        value
    }

    pub(crate) fn read_len_prefixed<'a>(data: &'a [u8], offset: &mut usize) -> &'a [u8] {
        let len = read_u32(data, offset) as usize;

        let bytes = &data[*offset..*offset + len];
        *offset += len;

        bytes
    }

    pub(crate) fn read_str(data: &[u8], offset: &mut usize) -> String {
        String::from_utf8_lossy(read_len_prefixed(data, offset)).into_owned()
    }

    pub(crate) fn read_opt_str(data: &[u8], offset: &mut usize) -> Option<String> {
        let flag = data[*offset];
        *offset += 1;

        if flag == 1 { Some(read_str(data, offset)) } else { None }
    }

    pub(crate) fn read_vec_u32(data: &[u8], offset: &mut usize) -> Vec<u32> {
        let len = read_u32(data, offset) as usize;
        let mut values = Vec::with_capacity(len);

        for _ in 0..len {
            values.push(read_u32(data, offset));
        }

        values
    }
}
