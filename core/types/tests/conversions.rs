// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::size_of;

use upac_abi::CONSTRAINT_ANY;
use upac_abi::hook::CProgressEvent;
use upac_abi::package::{CPackageDependency, CPackageMeta, CVersion};
use upac_abi::types::{COwned, CSlice};

use upac_types::error::ErrorKind;
use upac_types::package::{PackageDependency, PackageMeta, Version, VersionConstraint, VersionRequirement};
use upac_types::progress::ProgressEvent;

fn sample_version() -> Version {
    Version {
        epoch: 1,
        raw: "2.5.0-3~rc1".to_owned(),
    }
}

#[test]
fn version_c_round_trip_preserves_value() {
    let original = sample_version();

    let c_version = CVersion::from(original.clone());
    let restored = Version::try_from(&c_version).unwrap();

    assert_eq!(restored, original);
    unsafe { c_version.free() };
}

#[test]
fn package_meta_c_round_trip_preserves_value() {
    let original = PackageMeta {
        name: "upac".to_owned(),
        version: sample_version(),
        arch: "x86_64".to_owned(),
        arch_sub: None,
        maintainer: "JustPav".to_owned(),
        description: "package manager".to_owned(),
        license: Some("GPL-3.0-only".to_owned()),
        url: None,
        sha256: [7; 32],
        installed_size: 4096,
    };

    let c_meta = CPackageMeta::from(original.clone());
    let restored = PackageMeta::try_from(&c_meta).unwrap();

    assert_eq!(restored.name, original.name);
    assert_eq!(restored.version, original.version);
    assert_eq!(restored.arch, original.arch);
    assert_eq!(restored.arch_sub, original.arch_sub);
    assert_eq!(restored.maintainer, original.maintainer);
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.license, original.license);
    assert_eq!(restored.url, original.url);
    assert_eq!(restored.sha256, original.sha256);
    assert_eq!(restored.installed_size, original.installed_size);

    unsafe { c_meta.free() };
}

#[test]
fn a_dependency_on_any_version_round_trips() {
    let original = PackageDependency {
        name: "glibc".to_owned(),
        requirement: VersionRequirement::Any,
    };

    let c_dependency = CPackageDependency::from(original.clone());
    let restored = PackageDependency::try_from(&c_dependency).unwrap();

    assert_eq!(c_dependency.constraint, CONSTRAINT_ANY);
    assert_eq!(restored, original);
    unsafe { c_dependency.free() };
}

#[test]
fn every_bounded_dependency_round_trips() {
    let constraints = [
        VersionConstraint::Less,
        VersionConstraint::LessOrEqual,
        VersionConstraint::Equal,
        VersionConstraint::NotEqual,
        VersionConstraint::GreaterOrEqual,
        VersionConstraint::Greater,
    ];

    for constraint in constraints {
        let original = PackageDependency {
            name: "glibc".to_owned(),
            requirement: VersionRequirement::Bounded {
                constraint,
                version: sample_version(),
            },
        };

        let c_dependency = CPackageDependency::from(original.clone());
        let restored = PackageDependency::try_from(&c_dependency).unwrap();

        assert_eq!(restored, original);
        unsafe { c_dependency.free() };
    }
}

#[test]
fn a_bounded_dependency_without_a_version_is_rejected() {
    let c_dependency = CPackageDependency {
        struct_size: size_of::<CPackageDependency>(),
        name: CSlice::from_owned(b"glibc".to_vec()),
        constraint: 0b010,
        version: CVersion {
            struct_size: size_of::<CVersion>(),
            epoch: 0,
            raw: CSlice::from_slice(None),
        },
    };

    assert_eq!(
        PackageDependency::try_from(&c_dependency).err(),
        Some(ErrorKind::InvalidEntry)
    );
    unsafe { c_dependency.free() };
}

#[test]
fn version_constraint_rejects_the_any_and_empty_orderings() {
    assert_eq!(VersionConstraint::from_orderings(true, true, true), None);
    assert_eq!(VersionConstraint::from_orderings(false, false, false), None);
}

#[test]
fn a_progress_event_keeps_its_subject_on_the_c_side() {
    let c_event = CProgressEvent::from(ProgressEvent {
        stage: 3,
        subject: Some("glibc"),
        current: 1,
        total: 4,
    });

    assert_eq!(c_event.stage, 3);
    assert_eq!(unsafe { c_event.subject.as_str() }, Ok("glibc"));
    assert_eq!((c_event.current, c_event.total), (1, 4));
    unsafe { c_event.free() };
}

#[test]
fn a_progress_event_without_a_subject_has_a_null_subject() {
    let c_event = CProgressEvent::from(ProgressEvent {
        stage: 0,
        subject: None,
        current: 0,
        total: 0,
    });

    assert!(c_event.subject.ptr.is_null());
}
