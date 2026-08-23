// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

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
