// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Cursor;

use upac_decoder_rpm::header::{self, Header};
use upac_decoder_rpm::rpm::{NAME_TAG, POSTIN_TAG, POSTUN_TAG, PREIN_TAG, PREUN_TAG};
use upac_decoder_rpm::triggers;

enum RawValue<'a> {
    Str(&'a str),
}

fn build_header(entries: &[(u32, RawValue)]) -> Header {
    let mut index_bytes = Vec::new();
    let mut data_block: Vec<u8> = Vec::new();

    for (tag, value) in entries {
        let offset = data_block.len() as u32;
        let count: u32 = match value {
            RawValue::Str(text) => {
                data_block.extend_from_slice(text.as_bytes());
                data_block.push(0);
                1
            }
        };

        index_bytes.extend_from_slice(&tag.to_be_bytes());
        index_bytes.extend_from_slice(&0u32.to_be_bytes());
        index_bytes.extend_from_slice(&offset.to_be_bytes());
        index_bytes.extend_from_slice(&count.to_be_bytes());
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0xED, 0xAB, 0xEE, 0xDB]);
    bytes.resize(96, 0);

    bytes.extend_from_slice(&section_header(0, 0));
    bytes.extend_from_slice(&section_header(entries.len() as u32, data_block.len() as u32));
    bytes.extend_from_slice(&index_bytes);
    bytes.extend_from_slice(&data_block);

    let mut cursor = Cursor::new(bytes);
    header::read(&mut cursor).unwrap()
}

fn section_header(tag_count: u32, data_size: u32) -> [u8; 16] {
    let mut section = [0u8; 16];
    section[0..3].copy_from_slice(&[0x8E, 0xAD, 0xE8]);
    section[3] = 0x01;
    section[8..12].copy_from_slice(&tag_count.to_be_bytes());
    section[12..16].copy_from_slice(&data_size.to_be_bytes());
    section
}

#[test]
fn finds_no_triggers_when_no_scriptlets_are_present() {
    let header = build_header(&[]);

    let triggers = triggers::scan(&header);

    assert!(triggers.is_empty());
}

#[test]
fn a_bare_prein_covers_both_install_and_upgrade_positions() {
    let header = build_header(&[(PREIN_TAG, RawValue::Str("echo hi"))]);

    let triggers = triggers::scan(&header);

    assert_eq!(triggers, vec!["pre"]);
}

#[test]
fn finds_all_four_scriptlets() {
    let header = build_header(&[
        (PREIN_TAG, RawValue::Str("a")),
        (POSTIN_TAG, RawValue::Str("b")),
        (PREUN_TAG, RawValue::Str("c")),
        (POSTUN_TAG, RawValue::Str("d")),
    ]);

    let mut triggers = triggers::scan(&header);
    triggers.sort();

    assert_eq!(triggers, vec!["post", "postun", "pre", "preun"]);
}

#[test]
fn ignores_tags_that_are_not_scriptlets() {
    let header = build_header(&[(NAME_TAG, RawValue::Str("foo"))]);

    let triggers = triggers::scan(&header);

    assert!(triggers.is_empty());
}
