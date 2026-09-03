// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac::database::attribution::FileAttribute;
use upac::database::error::DatabaseError;
use upac::database::files::{FileStore, FileStoreMut};
use upac::database::meta::{MetaStore, MetaStoreMut};
use upac::database::triggers::{TriggerStore, TriggerStoreMut};
use upac::database::{InMemory, MemoryDatabase};

use upac_types::{DeclarativeTrigger, FileEntry, FileEntryScope, PackageMeta};

fn sample_meta(name: &str) -> PackageMeta {
    PackageMeta {
        name: name.to_owned(),
        arch: "x86_64".to_owned(),
        maintainer: "JustPav".to_owned(),
        description: "a package".to_owned(),
        installed_size: 100,
        ..Default::default()
    }
}

fn sample_file(path: &str, is_user: bool) -> FileEntry {
    FileEntry {
        path: path.to_owned(),
        is_user,
        scope: FileEntryScope::Prefix,
    }
}

#[test]
fn memory_database_bytes_round_trip_preserves_inserted_data() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();

    let bytes = db.into_bytes().unwrap();
    let reopened = MemoryDatabase::open_in_memory(bytes).unwrap();

    let meta = reopened.get_package_meta(uuid).unwrap().unwrap();
    assert_eq!(meta.name, "upac");
}

#[test]
fn insert_and_find_package_meta_by_identity() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();

    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    let found = db.find_package_uuid("upac", "x86_64", None).unwrap();

    assert_eq!(found, Some(uuid));
}

#[test]
fn find_package_uuid_is_none_for_an_unknown_package() {
    let db = MemoryDatabase::new_in_memory().unwrap();

    assert_eq!(db.find_package_uuid("missing", "x86_64", None).unwrap(), None);
}

#[test]
fn list_packages_metas_returns_every_inserted_package() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_meta(&sample_meta("upac-cli")).unwrap();

    let mut names: Vec<String> = db
        .list_packages_metas()
        .unwrap()
        .into_iter()
        .map(|meta| meta.name)
        .collect();
    names.sort();

    assert_eq!(names, vec!["upac".to_owned(), "upac-cli".to_owned()]);
}

#[test]
fn update_package_meta_replaces_the_stored_value() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    db.insert_package_meta(&sample_meta("upac")).unwrap();

    let mut updated = sample_meta("upac");
    updated.installed_size = 999;
    db.update_package_meta(&updated).unwrap();

    let uuid = db.find_package_uuid("upac", "x86_64", None).unwrap().unwrap();
    assert_eq!(db.get_package_meta(uuid).unwrap().unwrap().installed_size, 999);
}

#[test]
fn update_package_meta_fails_when_package_is_unknown() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();

    assert_eq!(
        db.update_package_meta(&sample_meta("missing")),
        Err(DatabaseError::PackageNotFound)
    );
}

#[test]
fn remove_package_meta_deletes_it_from_both_lookups() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    db.insert_package_meta(&sample_meta("upac")).unwrap();

    let removed = db.remove_package_meta("upac", "x86_64", None).unwrap();

    assert_eq!(removed.name, "upac");
    assert_eq!(db.find_package_uuid("upac", "x86_64", None).unwrap(), None);
    assert!(db.list_packages_metas().unwrap().is_empty());
}

#[test]
fn insert_and_find_file_owner() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();

    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    assert_eq!(db.find_file_owner("/etc/upac.toml").unwrap(), Some(uuid));
    assert_eq!(db.find_file_owner("/etc/missing").unwrap(), None);
}

#[test]
fn list_package_files_and_list_files_see_the_same_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    let owned = db.list_package_files(uuid).unwrap();
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].path, "/etc/upac.toml");

    let all = db.list_files().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].0, uuid);
}

#[test]
fn insert_package_file_does_not_clobber_an_already_user_owned_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();

    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", true))
        .unwrap();

    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    let files = db.list_package_files(uuid).unwrap();
    assert!(files[0].is_user, "a package reinsert must not clear user ownership");
}

#[test]
fn remove_package_file_rejects_a_user_owned_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", true))
        .unwrap();

    assert_eq!(
        db.remove_package_file(uuid, "/etc/upac.toml").unwrap_err(),
        DatabaseError::AccessDenied
    );
}

#[test]
fn remove_user_file_rejects_a_package_owned_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    assert_eq!(
        db.remove_user_file(uuid, "/etc/upac.toml").unwrap_err(),
        DatabaseError::AccessDenied
    );
}

#[test]
fn remove_package_file_deletes_a_package_owned_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    let removed = db.remove_package_file(uuid, "/etc/upac.toml").unwrap();

    assert_eq!(removed.path, "/etc/upac.toml");
    assert_eq!(db.find_file_owner("/etc/upac.toml").unwrap(), None);
}

#[test]
fn set_get_and_remove_declarative_triggers() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();

    assert!(db.get_declarative_triggers(uuid).unwrap().is_none());

    let trigger = DeclarativeTrigger {
        format: "deb".to_owned(),
        triggers: vec!["postinstall".to_owned()],
    };
    db.set_declarative_triggers(uuid, &trigger).unwrap();

    let stored = db.get_declarative_triggers(uuid).unwrap().unwrap();
    assert_eq!(stored.format, "deb");
    assert_eq!(stored.triggers, vec!["postinstall".to_owned()]);

    db.remove_declarative_triggers(uuid).unwrap();
    assert!(db.get_declarative_triggers(uuid).unwrap().is_none());
}

#[test]
fn attribute_file_resolves_the_owning_package_and_entry() {
    let mut db = MemoryDatabase::new_in_memory().unwrap();
    let uuid = db.insert_package_meta(&sample_meta("upac")).unwrap();
    db.insert_package_file(uuid, &sample_file("/etc/upac.toml", false))
        .unwrap();

    let attribution = db.attribute_file("/etc/upac.toml").unwrap().unwrap();

    assert_eq!(attribution.package_meta.name, "upac");
    assert_eq!(attribution.file_entry.path, "/etc/upac.toml");
}

#[test]
fn attribute_file_is_none_for_an_untracked_path() {
    let db = MemoryDatabase::new_in_memory().unwrap();

    assert!(db.attribute_file("/etc/missing").unwrap().is_none());
}
