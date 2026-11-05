// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;

use composefs::generic_tree::Stat;

use upac_composefs::tree::builder::TreeBuilder;

use upac_types::decoder::{PackageTrigger, PackageTriggers, TriggerPosition};
use upac_types::package::PackageInfo;

use upac_database::PackageId;
use upac_database::trigger::PackageTriggerStore;

fn package() -> PackageId {
    PackageId::new(&PackageInfo {
        name: "vim".to_owned(),
        arch: "x86_64".to_owned(),
        arch_sub: None,
    })
}

fn builder_with_package() -> TreeBuilder {
    let mut builder = TreeBuilder::new();
    builder.insert_dir("pkg", Stat::uninitialized()).unwrap();
    builder
        .insert_dir(format!("pkg/{}", package()), Stat::uninitialized())
        .unwrap();

    builder
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

#[test]
fn stored_triggers_read_back() {
    let mut builder = builder_with_package();

    PackageTriggerStore::write(&mut builder)
        .set(&package(), &triggers())
        .unwrap();
    let tree = builder.commit();

    assert_eq!(
        PackageTriggerStore::read(&tree).get(&package()).unwrap(),
        Some(triggers())
    );
}

#[test]
fn triggers_are_readable_lines_in_an_xattr() {
    let mut builder = builder_with_package();

    PackageTriggerStore::write(&mut builder)
        .set(&package(), &triggers())
        .unwrap();
    let tree = builder.commit();
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
    let tree = builder_with_package().commit();

    assert_eq!(PackageTriggerStore::read(&tree).get(&package()).unwrap(), None);
}

#[test]
fn removed_triggers_are_gone() {
    let mut builder = builder_with_package();
    let mut store = PackageTriggerStore::write(&mut builder);

    store.set(&package(), &triggers()).unwrap();
    store.remove(&package()).unwrap();
    let tree = builder.commit();

    assert_eq!(PackageTriggerStore::read(&tree).get(&package()).unwrap(), None);
}

#[test]
fn triggers_need_the_package_directory() {
    let mut builder = TreeBuilder::new();

    assert!(
        PackageTriggerStore::write(&mut builder)
            .set(&package(), &triggers())
            .is_err()
    );
}

#[test]
fn a_trigger_name_with_a_line_break_is_rejected() {
    let mut builder = builder_with_package();
    let broken = PackageTriggers {
        format: "deb".to_owned(),
        triggers: vec![PackageTrigger {
            position: TriggerPosition::PreInstall,
            name: "first\nsecond".to_owned(),
        }],
    };

    assert!(
        PackageTriggerStore::write(&mut builder)
            .set(&package(), &broken)
            .is_err()
    );
}
