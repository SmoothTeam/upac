// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::fat_label;

#[test]
fn fat_label_pads_short_labels_with_trailing_spaces() {
    let label = fat_label("esp");

    assert_eq!(&label[..3], b"ESP");
    assert_eq!(&label[3..], [b' '; 8]);
}

#[test]
fn fat_label_uppercases_input() {
    let label = fat_label("boot");

    assert_eq!(&label[..4], b"BOOT");
    assert_eq!(&label[4..], [b' '; 7]);
}

#[test]
fn fat_label_truncates_labels_longer_than_eleven_bytes() {
    let label = fat_label("a-very-long-label");

    assert_eq!(&label, b"A-VERY-LONG");
}
