// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::*;

fn sample_version() -> Version {
    Version {
        epoch: 1,
        parts: vec![2, 5, 0],
        pre: Some("rc1".to_owned()),
        release: 3,
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
fn file_entry_redb_round_trip_preserves_value() {
    let original = FileEntry {
        path: "/usr/bin/up".to_owned(),
        is_user: false,
    };

    let mut buf = Vec::new();
    FileEntry::encode_into(&mut buf, &original);

    let mut offset = 0;
    let restored = FileEntry::decode_from(&buf, &mut offset);

    assert_eq!(restored.path, original.path);
    assert_eq!(restored.is_user, original.is_user);
    assert_eq!(offset, buf.len());
}
