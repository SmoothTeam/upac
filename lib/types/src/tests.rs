// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::*;

fn sample_version() -> Version {
    Version {
        epoch: 1,
        raw: "2.5.0-3~rc1".to_owned(),
    }
}

#[test]
fn version_redb_round_trip_preserves_value() {
    let original = sample_version();

    let mut buf = Vec::new();
    Version::encode_into(&mut buf, &original);

    let mut offset = 0;
    let restored = Version::decode_from(&buf, &mut offset);

    assert_eq!(restored, original);
    assert_eq!(offset, buf.len());
}

#[test]
fn version_ord_equal_versions_compare_equal() {
    let a = Version {
        epoch: 0,
        raw: "1.2.3".to_owned(),
    };
    let b = a.clone();

    assert_eq!(a.cmp(&b), Ordering::Equal);
}

#[test]
fn version_ord_epoch_dominates_everything_else() {
    let low_epoch = Version {
        epoch: 0,
        raw: "99.99.99".to_owned(),
    };
    let high_epoch = Version {
        epoch: 1,
        raw: "0.0.1".to_owned(),
    };

    assert!(high_epoch > low_epoch);
}

#[test]
fn version_ord_numeric_segments_compare_numerically() {
    let a = Version {
        epoch: 0,
        raw: "1.9".to_owned(),
    };
    let b = Version {
        epoch: 0,
        raw: "1.10".to_owned(),
    };

    assert!(b > a);
}

#[test]
fn version_ord_numeric_beats_alpha_at_same_position() {
    let release = Version {
        epoch: 0,
        raw: "1.0".to_owned(),
    };
    let pre_release = Version {
        epoch: 0,
        raw: "1.0a".to_owned(),
    };

    assert!(release > pre_release);
}

#[test]
fn version_ord_trailing_extra_numeric_is_newer() {
    let a = Version {
        epoch: 0,
        raw: "1.0".to_owned(),
    };
    let b = Version {
        epoch: 0,
        raw: "1.0.1".to_owned(),
    };

    assert!(b > a);
}

#[test]
fn version_ord_trailing_extra_alpha_is_older() {
    let a = Version {
        epoch: 0,
        raw: "1.0".to_owned(),
    };
    let b = Version {
        epoch: 0,
        raw: "1.0-alpha".to_owned(),
    };

    assert!(b < a);
}

#[test]
fn version_ord_mixed_format_examples_compare_consistently() {
    let semver = Version {
        epoch: 0,
        raw: "1.23".to_owned(),
    };
    let calver_dotted = Version {
        epoch: 0,
        raw: "26.5.4".to_owned(),
    };
    let calver_flat = Version {
        epoch: 0,
        raw: "20263545".to_owned(),
    };
    let alpha_mixed = Version {
        epoch: 0,
        raw: "1.13pre-1".to_owned(),
    };
    let no_suffix = Version {
        epoch: 0,
        raw: "1.13".to_owned(),
    };

    assert!(calver_dotted > semver);
    assert!(calver_flat > calver_dotted);
    assert_eq!(alpha_mixed.cmp(&alpha_mixed.clone()), Ordering::Equal);
    assert!(no_suffix > alpha_mixed);
}

#[test]
fn file_entry_redb_round_trip_preserves_value() {
    let original = FileEntry {
        path: "/usr/bin/up".to_owned(),
        is_user: false,
        scope: FileEntryScope::Prefix,
    };

    let mut buf = Vec::new();
    FileEntry::encode_into(&mut buf, &original);

    let mut offset = 0;
    let restored = FileEntry::decode_from(&buf, &mut offset);

    assert_eq!(restored.path, original.path);
    assert_eq!(restored.is_user, original.is_user);
    assert_eq!(restored.scope, original.scope);
    assert_eq!(offset, buf.len());
}
