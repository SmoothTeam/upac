use libmdbx::{RW, TransactionKind};
use uuid::Uuid;

use super::{DatabaseTransaction, Store};

use crate::types::PackageMeta;
use crate::types::errors::DatabaseError;

pub fn insert(transaction: &DatabaseTransaction<'_, RW>, meta: &PackageMeta) -> Result<Uuid, DatabaseError> {
    let uuid = Uuid::new_v4();
    let bytes = rmp_serde::to_vec(meta)?;

    transaction.put(Store::Packages, uuid, &bytes)?;

    Ok(uuid)
}

pub fn delete(
    transaction: &DatabaseTransaction<'_, RW>, name: &str, arch: &str, arch_sub: Option<&str>,
) -> Result<(), DatabaseError> {
    let found = exists(transaction, name, arch, arch_sub)?.ok_or(DatabaseError::PackageNotFound)?;

    transaction.delete(Store::Packages, found)?;

    Ok(())
}

pub fn update(transaction: &DatabaseTransaction<'_, RW>, package_meta: &PackageMeta) -> Result<(), DatabaseError> {
    let found = exists(
        transaction,
        &package_meta.name,
        &package_meta.arch,
        package_meta.arch_sub.as_deref(),
    )?
    .ok_or(DatabaseError::PackageNotFound)?;

    let package_meta_as_bytes = rmp_serde::to_vec(package_meta)?;

    transaction.put(Store::Packages, found, &package_meta_as_bytes)?;

    Ok(())
}

pub fn exists<K: TransactionKind>(
    transaction: &DatabaseTransaction<'_, K>, name: &str, arch: &str, arch_sub: Option<&str>,
) -> Result<Option<Uuid>, DatabaseError> {
    for (uuid, value) in transaction.entries(Store::Packages)? {
        let meta: PackageMeta = rmp_serde::from_slice(&value)?;

        if meta.name == name && meta.arch == arch && meta.arch_sub.as_deref() == arch_sub {
            return Ok(Some(uuid));
        }
    }
    Ok(None)
}

pub fn list<K: TransactionKind>(transaction: &DatabaseTransaction<'_, K>) -> Result<Vec<PackageMeta>, DatabaseError> {
    let mut out = Vec::new();

    for (_uuid, value) in transaction.entries(Store::Packages)? {
        out.push(rmp_serde::from_slice(&value)?);
    }

    Ok(out)
}
