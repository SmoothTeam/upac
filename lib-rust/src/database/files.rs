use libmdbx::{RW, TransactionKind};
use uuid::Uuid;

use super::{DatabaseTransaction, Store};

use crate::errors::DatabaseError;
use crate::types::FileEntry;

enum PathScan {
    Deleted,
    UserFile,
    NotFound,
}

fn take_path(txn: &DatabaseTransaction<'_, RW>, uuid: Uuid, path: &str) -> Result<PathScan, DatabaseError> {
    for value in txn.values_of(Store::Files, uuid)? {
        let file_entry: FileEntry = rmp_serde::from_slice(&value)?;

        if file_entry.path == path {
            if file_entry.is_user {
                return Ok(PathScan::UserFile);
            }
            txn.delete_value(Store::Files, uuid, &value)?;

            return Ok(PathScan::Deleted);
        }
    }

    Ok(PathScan::NotFound)
}

pub fn insert(txn: &DatabaseTransaction<'_, RW>, uuid: Uuid, file_entry: &FileEntry) -> Result<(), DatabaseError> {
    if let PathScan::UserFile = take_path(txn, uuid, &file_entry.path)? {
        return Ok(());
    }

    let serialized_file_entry = rmp_serde::to_vec(file_entry)?;

    txn.put(Store::Files, uuid, &serialized_file_entry)?;

    Ok(())
}

pub fn delete(txn: &DatabaseTransaction<'_, RW>, uuid: Uuid, file_path: &str) -> Result<(), DatabaseError> {
    take_path(txn, uuid, file_path)?;

    Ok(())
}

pub fn update(txn: &DatabaseTransaction<'_, RW>, uuid: Uuid, file_entry: &FileEntry) -> Result<(), DatabaseError> {
    if let PathScan::UserFile = take_path(txn, uuid, &file_entry.path)? {
        return Ok(());
    }

    let serialized_file_entry = rmp_serde::to_vec(file_entry)?;

    txn.put(Store::Files, uuid, &serialized_file_entry)?;

    Ok(())
}

pub fn exists<K: TransactionKind>(
    txn: &DatabaseTransaction<'_, K>, uuid: Uuid, file_path: &str,
) -> Result<bool, DatabaseError> {
    for value in txn.values_of(Store::Files, uuid)? {
        let file_entry: FileEntry = rmp_serde::from_slice(&value)?;

        if file_entry.path == file_path {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn list<K: TransactionKind>(txn: &DatabaseTransaction<'_, K>, uuid: Uuid) -> Result<Vec<FileEntry>, DatabaseError> {
    let mut out_files_list = Vec::new();

    for value in txn.values_of(Store::Files, uuid)? {
        out_files_list.push(rmp_serde::from_slice(&value)?);
    }

    Ok(out_files_list)
}
