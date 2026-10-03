// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use redb::{ReadableDatabase, ReadableTable};

use twox_hash::xxhash3_64::Hasher as XxHasher;

use uuid::Uuid;

use upac_types::package::PackageMeta;

use super::error::DatabaseError;
use super::{
    MemoryDatabase, PACKAGES_HASH_TABLE, PACKAGES_UUID_TABLE, ReadTransactionExt, ReadableSource, record_decode,
    record_encode,
};

pub trait MetaStore {
    fn identity_hash(name: &str, arch: &str, arch_sub: Option<&str>) -> Result<u64, DatabaseError> {
        Ok(XxHasher::oneshot(&record_encode(&(name, arch, arch_sub))?))
    }

    fn lookup_uuid(
        by_name: &impl ReadableTable<u64, Uuid>, name: &str, arch: &str, arch_sub: Option<&str>,
    ) -> Result<Option<Uuid>, DatabaseError> {
        Ok(by_name
            .get(Self::identity_hash(name, arch, arch_sub)?)?
            .map(|guard| guard.value()))
    }

    fn find_package_uuid(&self, name: &str, arch: &str, arch_sub: Option<&str>) -> Result<Option<Uuid>, DatabaseError>;
    fn get_package_meta(&self, uuid: Uuid) -> Result<Option<PackageMeta>, DatabaseError>;
    fn list_packages_metas(&self) -> Result<Vec<PackageMeta>, DatabaseError>;
}

pub trait MetaStoreMut: MetaStore {
    fn insert_package_meta(&mut self, meta: &PackageMeta) -> Result<Uuid, DatabaseError>;
    fn update_package_meta(&mut self, meta: &PackageMeta) -> Result<(), DatabaseError>;
    fn remove_package_meta(
        &mut self, name: &str, arch: &str, arch_sub: Option<&str>,
    ) -> Result<PackageMeta, DatabaseError>;
}

impl<T: ReadableSource> MetaStore for T {
    fn find_package_uuid(&self, name: &str, arch: &str, arch_sub: Option<&str>) -> Result<Option<Uuid>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(by_name) = transaction.open_table_or_none(PACKAGES_HASH_TABLE)? else {
            return Ok(None);
        };

        Self::lookup_uuid(&by_name, name, arch, arch_sub)
    }

    fn get_package_meta(&self, uuid: Uuid) -> Result<Option<PackageMeta>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(packages) = transaction.open_table_or_none(PACKAGES_UUID_TABLE)? else {
            return Ok(None);
        };

        packages
            .get(uuid)?
            .map(|guard| record_decode(guard.value()))
            .transpose()
    }

    fn list_packages_metas(&self) -> Result<Vec<PackageMeta>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(packages) = transaction.open_table_or_none(PACKAGES_UUID_TABLE)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();

        for entry in packages.iter()? {
            let (_uuid, meta) = entry?;
            out.push(record_decode(meta.value())?);
        }

        Ok(out)
    }
}

impl MetaStoreMut for MemoryDatabase {
    fn insert_package_meta(&mut self, meta: &PackageMeta) -> Result<Uuid, DatabaseError> {
        let uuid = Uuid::new_v4();
        let transaction = self.database.begin_write()?;

        transaction
            .open_table(PACKAGES_UUID_TABLE)?
            .insert(uuid, record_encode(meta)?.as_slice())?;

        let hash = Self::identity_hash(&meta.name, &meta.arch, meta.arch_sub.as_deref())?;
        transaction.open_table(PACKAGES_HASH_TABLE)?.insert(hash, uuid)?;

        transaction.commit()?;
        Ok(uuid)
    }

    fn update_package_meta(&mut self, meta: &PackageMeta) -> Result<(), DatabaseError> {
        let transaction = self.database.begin_write()?;

        let by_name = transaction.open_table(PACKAGES_HASH_TABLE)?;
        let uuid = Self::lookup_uuid(&by_name, &meta.name, &meta.arch, meta.arch_sub.as_deref())?
            .ok_or(DatabaseError::PackageNotFound)?;

        transaction
            .open_table(PACKAGES_UUID_TABLE)?
            .insert(uuid, record_encode(meta)?.as_slice())?;

        drop(by_name);
        transaction.commit()?;
        Ok(())
    }

    fn remove_package_meta(
        &mut self, name: &str, arch: &str, arch_sub: Option<&str>,
    ) -> Result<PackageMeta, DatabaseError> {
        let transaction = self.database.begin_write()?;

        let mut by_name = transaction.open_table(PACKAGES_HASH_TABLE)?;
        let uuid = Self::lookup_uuid(&by_name, name, arch, arch_sub)?.ok_or(DatabaseError::PackageNotFound)?;

        by_name.remove(Self::identity_hash(name, arch, arch_sub)?)?;

        let mut packages = transaction.open_table(PACKAGES_UUID_TABLE)?;
        let removed = record_decode(packages.remove(uuid)?.ok_or(DatabaseError::PackageNotFound)?.value())?;

        drop(by_name);
        drop(packages);
        transaction.commit()?;

        Ok(removed)
    }
}
