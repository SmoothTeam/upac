use std::borrow::Cow;

use libmdbx::{RW, TransactionKind, WriteFlags};

use super::{DatabaseError, FILES_TABLE, Txn};

use crate::types::FileEntry;

enum PathScan {
    Deleted,
    UserFile,
    NotFound,
}

fn scan_and_clear_path(
    transaction: &Txn<'_, RW>,
    table: &libmdbx::Table<'_>,
    uuid: [u8; 16],
    path: &str,
) -> Result<PathScan, DatabaseError> {
    let mut cursor = transaction
        .inner
        .cursor(table)
        .map_err(|_| DatabaseError::ReadError)?;

    let positioned = cursor
        .set_lowerbound::<[u8; 16], Cow<[u8]>>(&uuid, None)
        .map_err(|_| DatabaseError::ReadError)?;

    let mut current = match positioned {
        Some((_next, key, value)) if key == uuid => Some(value),
        _ => return Ok(PathScan::NotFound),
    };

    while let Some(value) = current {
        let entry: FileEntry = rmp_serde::from_slice(value.as_ref())?;
        if entry.path == path {
            if entry.is_user {
                return Ok(PathScan::UserFile);
            }
            cursor
                .del(WriteFlags::CURRENT)
                .map_err(|_| DatabaseError::WriteError)?;
            return Ok(PathScan::Deleted);
        }
        current = cursor
            .next_dup::<[u8; 16], Cow<[u8]>>()
            .map_err(|_| DatabaseError::ReadError)?
            .map(|(_k, v)| v);
    }

    Ok(PathScan::NotFound)
}

pub fn insert(
    transaction: &Txn<'_, RW>,
    uuid: [u8; 16],
    file_entry: &FileEntry,
) -> Result<(), DatabaseError> {
    let table = transaction
        .inner
        .open_table(Some(FILES_TABLE))
        .map_err(|_| DatabaseError::PackageNotFound)?;

    if let PathScan::Protected = scan_and_clear_path(txn, &table, uuid, &file_entry.path)? {
        return Ok(()); // user file — leave it, write nothing
    }

    let serialized = rmp_serde::to_vec(file_entry)?;
    let mut cursor = txn
        .inner
        .cursor(&table)
        .map_err(|_| DatabaseError::ReadError)?;
    cursor
        .put(&uuid, &serialized, WriteFlags::UPSERT)
        .map_err(|_| DatabaseError::WriteError)?;
    Ok(())
}

/// Delete the (non-user) file entry with `file_path` under `uuid`.
/// User files are left in place; a missing entry is silently ignored.
pub fn delete(txn: &Txn<'_, RW>, uuid: [u8; 16], file_path: &str) -> Result<(), DatabaseError> {
    let table = txn
        .inner
        .open_table(Some(FILES_TABLE))
        .map_err(|_| DatabaseError::PackageNotFound)?;

    // Deleted / Protected / NotFound are all acceptable outcomes here.
    scan_and_clear_path(txn, &table, uuid, file_path)?;
    Ok(())
}

/// Replace the entry with `file_entry.path` under `uuid`: drop the stale
/// non-user duplicate (if any), then append the new value. User files protected.
pub fn update(
    txn: &Txn<'_, RW>,
    uuid: [u8; 16],
    file_entry: &FileEntry,
) -> Result<(), DatabaseError> {
    let table = txn
        .inner
        .open_table(Some(FILES_TABLE))
        .map_err(|_| DatabaseError::PackageNotFound)?;

    if let PathScan::Protected = scan_and_clear_path(txn, &table, uuid, &file_entry.path)? {
        return Ok(());
    }

    let serialized = rmp_serde::to_vec(file_entry)?;
    let mut cursor = txn
        .inner
        .cursor(&table)
        .map_err(|_| DatabaseError::ReadError)?;
    cursor
        .put(&uuid, &serialized, WriteFlags::UPSERT)
        .map_err(|_| DatabaseError::WriteError)?;
    Ok(())
}

/// True if `path` exists under `uuid` as a duplicate (any entry, user or not).
pub fn exists<K: TransactionKind>(
    txn: &Txn<'_, K>,
    uuid: [u8; 16],
    file_path: &str,
) -> Result<bool, DatabaseError> {
    let table = txn
        .inner
        .open_table(Some(FILES_TABLE))
        .map_err(|_| DatabaseError::PackageNotFound)?;

    let mut cursor = txn
        .inner
        .cursor(&table)
        .map_err(|_| DatabaseError::ReadError)?;

    let positioned = cursor
        .set_lowerbound::<[u8; 16], Cow<[u8]>>(&uuid, None)
        .map_err(|_| DatabaseError::ReadError)?;

    let mut current = match positioned {
        Some((_next, key, value)) if key == uuid => Some(value),
        _ => return Ok(false),
    };

    while let Some(value) = current {
        let entry: FileEntry = rmp_serde::from_slice(value.as_ref())?;
        if entry.path == file_path {
            return Ok(true);
        }
        current = cursor
            .next_dup::<[u8; 16], Cow<[u8]>>()
            .map_err(|_| DatabaseError::ReadError)?
            .map(|(_k, v)| v);
    }

    Ok(false)
}

/// Collect every `FileEntry` stored under `uuid`. (Zig: walk dups into a list.)
pub fn list<K: TransactionKind>(
    txn: &Txn<'_, K>,
    uuid: [u8; 16],
) -> Result<Vec<FileEntry>, DatabaseError> {
    let table = txn
        .inner
        .open_table(Some(FILES_TABLE))
        .map_err(|_| DatabaseError::PackageNotFound)?;

    let mut cursor = txn
        .inner
        .cursor(&table)
        .map_err(|_| DatabaseError::ReadError)?;

    let mut out = Vec::new();

    let positioned = cursor
        .set_lowerbound::<[u8; 16], Cow<[u8]>>(&uuid, None)
        .map_err(|_| DatabaseError::ReadError)?;

    let mut current = match positioned {
        Some((_next, key, value)) if key == uuid => Some(value),
        _ => return Ok(out), // no entries for this uuid
    };

    while let Some(value) = current {
        out.push(rmp_serde::from_slice(value.as_ref())?);
        current = cursor
            .next_dup::<[u8; 16], Cow<[u8]>>()
            .map_err(|_| DatabaseError::ReadError)?
            .map(|(_k, v)| v);
    }

    Ok(out)
}
