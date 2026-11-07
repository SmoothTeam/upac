// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use composefs::fsverity::compute_verity;
use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_types::decoder::{PackageTrigger, PackageTriggers, TriggerPosition};
use upac_types::package::PackageInfo;

use upac_database::package::PackageStore;
use upac_database::transaction::TransactionStore;
use upac_database::{PackageEntryId, PackageEntryKind, PackageId};

fn package(name: &str) -> PackageId {
    PackageId::new(&PackageInfo {
        name: name.to_owned(),
        arch: "x86_64".to_owned(),
        arch_sub: None,
    })
}

fn file_stat() -> Stat {
    Stat {
        st_mode: 0o755,
        ..Stat::uninitialized()
    }
}

fn system_builder() -> TreeBuilder {
    let mut builder = TreeBuilder::new();
    PackageStore::write(&mut builder).init().unwrap();
    TransactionStore::write(&mut builder).init().unwrap();

    for directory in ["cmp", "cmp/usr", "cmp/usr/bin", "cmp/usr/lib"] {
        builder.insert_dir(directory, Stat::uninitialized()).unwrap();
    }

    builder
        .insert(
            "cmp/usr/bin/tool",
            LeafContent::Regular(RegularFile::Inline(Box::from(&b"tool"[..]))),
            file_stat(),
        )
        .unwrap();

    builder
}

fn tool_entry() -> PackageEntryId {
    PackageEntryId::new(
        PackageEntryKind::File,
        compute_verity(b"tool"),
        &file_stat(),
        Path::new("cmp/usr/bin/tool"),
    )
}

fn lib_entry() -> PackageEntryId {
    PackageEntryId::new(
        PackageEntryKind::Dir,
        compute_verity(b"cmp/usr/lib"),
        &Stat::uninitialized(),
        Path::new("cmp/usr/lib"),
    )
}

fn triggers() -> PackageTriggers {
    PackageTriggers {
        format: "deb".to_owned(),
        triggers: vec![
            PackageTrigger {
                position: TriggerPosition::PostInstall,
                name: "ldconfig".to_owned(),
            },
            PackageTrigger {
                position: TriggerPosition::PostRemove,
                name: "update icon cache".to_owned(),
            },
        ],
    }
}

fn installed(entries: &[(PackageEntryId, &str)], package_triggers: Option<&PackageTriggers>) -> Tree {
    let mut builder = system_builder();
    let mut store = PackageStore::write(&mut builder);

    store.create(&package("vim")).unwrap();

    for (entry, place) in entries {
        store.insert(&package("vim"), entry, Path::new(place)).unwrap();
    }

    if let Some(package_triggers) = package_triggers {
        store.set_triggers(&package("vim"), package_triggers).unwrap();
    }

    builder.commit()
}

#[test]
fn an_initialised_store_lists_no_packages() {
    let tree = system_builder().commit();

    assert!(PackageStore::read(&tree).list().unwrap().is_empty());
}

#[test]
fn created_packages_are_listed_without_the_transaction_log() {
    let mut builder = system_builder();
    let mut store = PackageStore::write(&mut builder);
    store.create(&package("vim")).unwrap();
    store.create(&package("bash")).unwrap();
    let tree = builder.commit();

    let listed: HashSet<PackageId> = PackageStore::read(&tree).list().unwrap().into_iter().collect();

    assert_eq!(listed, HashSet::from([package("vim"), package("bash")]));
}

#[test]
fn a_file_entry_is_the_same_inode_as_its_place() {
    let tree = installed(&[(tool_entry(), "cmp/usr/bin/tool")], None);
    let filesystem = tree.filesystem();

    let (place_directory, place_name) = filesystem.root.split(OsStr::new("cmp/usr/bin/tool")).unwrap();
    let entry_path = format!("pkg/vim:x86_64/{}", tool_entry());
    let (entry_directory, entry_name) = filesystem.root.split(OsStr::new(&entry_path)).unwrap();

    assert_eq!(
        place_directory.leaf_id(place_name).unwrap(),
        entry_directory.leaf_id(entry_name).unwrap()
    );
}

#[test]
fn a_directory_entry_is_a_symlink_to_its_place() {
    let tree = installed(&[(lib_entry(), "cmp/usr/lib")], None);

    let content = tree.get(format!("pkg/vim:x86_64/{}", lib_entry())).unwrap();

    assert!(matches!(content, LeafContent::Symlink(target) if target.as_ref() == OsStr::new("/cmp/usr/lib")));
}

#[test]
fn entries_read_back_without_the_triggers_file() {
    let tree = installed(
        &[(tool_entry(), "cmp/usr/bin/tool"), (lib_entry(), "cmp/usr/lib")],
        Some(&triggers()),
    );

    let entries: HashSet<PackageEntryId> = PackageStore::read(&tree)
        .entries(&package("vim"))
        .unwrap()
        .into_iter()
        .collect();

    assert_eq!(entries, HashSet::from([tool_entry(), lib_entry()]));
}

#[test]
fn stored_triggers_read_back() {
    let tree = installed(&[], Some(&triggers()));

    assert_eq!(
        PackageStore::read(&tree).triggers(&package("vim")).unwrap(),
        Some(triggers())
    );
}

#[test]
fn triggers_are_readable_lines_in_an_xattr() {
    let tree = installed(&[], Some(&triggers()));
    let stat = tree.get_metadata("pkg/vim:x86_64/triggers").unwrap();

    assert_eq!(
        stat.xattrs.get(OsStr::new("user.upac.triggers")).unwrap().as_ref(),
        b"post-install ldconfig\npost-remove update icon cache\n"
    );
    assert_eq!(
        stat.xattrs.get(OsStr::new("user.upac.format")).unwrap().as_ref(),
        b"deb"
    );
}

#[test]
fn a_package_without_triggers_reads_as_none() {
    let tree = installed(&[], None);

    assert_eq!(PackageStore::read(&tree).triggers(&package("vim")).unwrap(), None);
}

#[test]
fn a_removed_package_is_gone_with_its_entries_and_triggers() {
    let mut builder = system_builder();
    let mut store = PackageStore::write(&mut builder);
    store.create(&package("vim")).unwrap();
    store
        .insert(&package("vim"), &tool_entry(), Path::new("cmp/usr/bin/tool"))
        .unwrap();
    store.set_triggers(&package("vim"), &triggers()).unwrap();
    store.remove(&package("vim")).unwrap();
    let tree = builder.commit();

    assert!(PackageStore::read(&tree).list().unwrap().is_empty());
    assert!(tree.check("cmp/usr/bin/tool"));
}

#[test]
fn package_data_needs_the_package_directory() {
    let mut builder = system_builder();
    let mut store = PackageStore::write(&mut builder);

    assert!(store.set_triggers(&package("vim"), &triggers()).is_err());
    assert!(
        store
            .insert(&package("vim"), &tool_entry(), Path::new("cmp/usr/bin/tool"))
            .is_err()
    );
}

#[test]
fn a_trigger_name_with_a_line_break_is_rejected() {
    let mut builder = system_builder();
    let mut store = PackageStore::write(&mut builder);
    store.create(&package("vim")).unwrap();
    let broken = PackageTriggers {
        format: "deb".to_owned(),
        triggers: vec![PackageTrigger {
            position: TriggerPosition::PreInstall,
            name: "first\nsecond".to_owned(),
        }],
    };

    assert!(store.set_triggers(&package("vim"), &broken).is_err());
}
