// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Cursor;

use upac_types::decoder::read_to_string;

#[test]
fn read_to_string_returns_the_full_utf8_content() {
    let mut reader = Cursor::new(b"hello world".to_vec());

    assert_eq!(read_to_string(&mut reader).unwrap(), "hello world");
}

#[test]
fn read_to_string_rejects_invalid_utf8() {
    let mut reader = Cursor::new(vec![0xFF, 0xFE, 0xFD]);

    assert!(read_to_string(&mut reader).is_err());
}
