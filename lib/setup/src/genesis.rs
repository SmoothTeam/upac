// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::env::temp_dir;
use std::fs::{File, create_dir_all, write};
use std::io::Read;
use std::path::{Path, PathBuf};

use composefs::erofs::reader::erofs_to_filesystem;
use composefs::fsverity::FsVerityHashValue;
use composefs::generic_tree::Stat;
use composefs::repository::{ImportContext, Repository};
use composefs::tree::FileSystem;

use upac::boot::write_boot_entry;
use upac::composefs::file::FileHandle;
use upac::composefs::repository::{ObjectID, commit_tree};
use upac::database::files::FileStoreMut;
use upac::database::meta::MetaStoreMut;
use upac::database::record::DeployRecord;
use upac::database::{InMemory, MemoryDatabase};
use upac::layout::boot_plugins::{BOOT_PLUGINS_DIR, MANIFEST_EXTENSION};
use upac::layout::database::DATABASE_PATH;
use upac::plugin::boot::resolve_boot_plugin;

use upac_types::{FileEntry, FileEntryScope, PackageMeta};

use crate::data::SetupExistingData;
use crate::error::SetupError;
use crate::layout::genesis::SCRATCH_FILENAME;
use crate::meta::SourceDir;
use crate::target::TargetSysroot;

pub fn run(data: &SetupExistingData) -> Result<(), SetupError> {
    let target = TargetSysroot::new(
        Path::new(data.deploy_device),
        data.deploy_fs,
        Path::new(data.esp_device),
        PathBuf::from(data.mount_point()),
        &data.extra_mounts,
    )?;

    let meta = read_meta(data)?;
    let (prefix_digest, config_digest) = import_trees(&target, data, meta)?;

    write_deploy_record(&target, data, &prefix_digest, &config_digest)?;
    stage_boot(&target, data, &prefix_digest)?;

    Ok(())
}

fn read_meta(data: &SetupExistingData) -> Result<PackageMeta, SetupError> {
    let source = SourceDir {
        path: Path::new(data.source_dir),
    };

    let mut meta = source.read(data.meta_filename)?;
    let (sha256, installed_size) = source.checksum(!data.empty_config)?;
    meta.sha256 = sha256;
    meta.installed_size = installed_size;

    Ok(meta)
}

fn import_trees(
    target: &TargetSysroot, data: &SetupExistingData, meta: PackageMeta,
) -> Result<(ObjectID, ObjectID), SetupError> {
    let repository = target.repository();
    let mut import_ctx = ImportContext::default();

    let mut prefix_tree = FileSystem::new(Stat::uninitialized());
    let usr_source = Path::new(data.source_dir).join("usr");
    let imported = if usr_source.is_dir() {
        FileHandle::new(PathBuf::new()).import_directory(
            repository,
            &mut prefix_tree,
            &usr_source,
            &mut import_ctx,
            data.cancel_token,
        )?
    } else {
        Vec::new()
    };

    let mut config_tree = FileSystem::new(Stat::uninitialized());
    let config_source = Path::new(data.source_dir).join("etc");
    let imported_config = if !data.empty_config && config_source.is_dir() {
        FileHandle::new(PathBuf::new()).import_directory(
            repository,
            &mut config_tree,
            &config_source,
            &mut import_ctx,
            data.cancel_token,
        )?
    } else {
        Vec::new()
    };

    let mut database = MemoryDatabase::new_in_memory()?;
    let uuid = database.insert_package_meta(&meta)?;

    for path in imported {
        database.insert_package_file(
            uuid,
            &FileEntry {
                path: path.to_string_lossy().into_owned(),
                is_user: false,
                scope: FileEntryScope::Prefix,
            },
        )?;
    }
    for path in imported_config {
        database.insert_package_file(
            uuid,
            &FileEntry {
                path: path.to_string_lossy().into_owned(),
                is_user: false,
                scope: FileEntryScope::Config,
            },
        )?;
    }

    let database_bytes = database.into_bytes()?;
    let database_scratch_path = temp_dir().join(SCRATCH_FILENAME);
    write(&database_scratch_path, &database_bytes)?;

    FileHandle::new(DATABASE_PATH).insert_file(
        repository,
        &mut prefix_tree,
        &File::open(&database_scratch_path)?,
        Stat::uninitialized(),
        &mut import_ctx,
    )?;

    let prefix_digest = commit_tree(repository, prefix_tree)?;
    let config_digest = commit_tree(repository, config_tree)?;

    Ok((prefix_digest, config_digest))
}

fn write_deploy_record(
    target: &TargetSysroot, data: &SetupExistingData, prefix_digest: &ObjectID, config_digest: &ObjectID,
) -> Result<(), SetupError> {
    let prefix_digest_hex = prefix_digest.to_hex();
    let deploy_dir = target.deploy_dir(&prefix_digest_hex);
    create_dir_all(&deploy_dir)?;

    let record = DeployRecord {
        prefix_digest: prefix_digest_hex,
        subject: "genesis".to_owned(),
        message: None,
        seq: DeployRecord::allocate_seq(&target.next_seq_path())?,
        timestamp: DeployRecord::now_secs(),
        config_history: Vec::new(),
        working_config: config_digest.to_hex(),
        pinned: data.pinned,
    };
    record.write(&deploy_dir)?;

    Ok(())
}

fn stage_boot(target: &TargetSysroot, data: &SetupExistingData, prefix_digest: &ObjectID) -> Result<(), SetupError> {
    let repository = target.repository();
    let prefix_digest_hex = prefix_digest.to_hex();

    let prefix_tree = reopen_tree(repository, &prefix_digest_hex)?;
    let entry_name = write_boot_entry(
        repository,
        &prefix_tree,
        prefix_digest.clone(),
        &target.esp_mount_point(),
        &prefix_digest_hex,
    )?;

    let plugin = resolve_boot_plugin(BOOT_PLUGINS_DIR, MANIFEST_EXTENSION, data.boot_plugin)?;
    plugin.set_one_shot(&entry_name)?;

    Ok(())
}

fn reopen_tree(repository: &Repository<ObjectID>, digest: &str) -> Result<FileSystem<ObjectID>, SetupError> {
    let (image, _enable_verity) = repository.open_image(digest)?;

    let mut data = Vec::new();
    File::from(image).read_to_end(&mut data)?;

    Ok(erofs_to_filesystem(&data)?)
}
