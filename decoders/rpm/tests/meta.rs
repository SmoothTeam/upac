// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::Cursor;

use upac_abi::decoder::DecodeError;
use upac_abi::decoder::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS};
use upac_decoder_rpm::header::{self, Header};
use upac_decoder_rpm::rpm::{
    ARCH_TAG, LICENSE_TAG, NAME_TAG, PACKAGER_TAG, RELEASE_TAG, REQUIRE_FLAGS_TAG, REQUIRE_NAME_TAG,
    REQUIRE_VERSION_TAG, SIZE_TAG, SUMMARY_TAG, URL_TAG, VERSION_TAG,
};
use upac_types::decoder::DecodeMeta;

const CHECKSUM: [u8; 32] = [7; 32];

enum RawValue<'a> {
    Str(&'a str),
    StrArray(&'a [&'a str]),
    I32(i32),
    I32Array(&'a [i32]),
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
            RawValue::StrArray(items) => {
                for item in *items {
                    data_block.extend_from_slice(item.as_bytes());
                    data_block.push(0);
                }
                items.len() as u32
            }
            RawValue::I32(value) => {
                data_block.extend_from_slice(&value.to_be_bytes());
                1
            }
            RawValue::I32Array(items) => {
                for item in *items {
                    data_block.extend_from_slice(&item.to_be_bytes());
                }
                items.len() as u32
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
fn parses_minimal_meta_with_defaults() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (ARCH_TAG, RawValue::Str("x86_64")),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.name, "foo");
    assert_eq!(decoded.meta.version.raw, "1.2.3");
    assert_eq!(decoded.meta.version.epoch, 0);
    assert_eq!(decoded.meta.arch, "x86_64");
    assert_eq!(decoded.meta.maintainer, "");
    assert_eq!(decoded.meta.description, "");
    assert_eq!(decoded.meta.license, None);
    assert_eq!(decoded.meta.url, None);
    assert_eq!(decoded.meta.installed_size, 0);
    assert_eq!(decoded.meta.sha256, CHECKSUM);
    assert!(decoded.dependencies.is_empty());
}

#[test]
fn release_is_joined_into_raw_version() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (RELEASE_TAG, RawValue::Str("2.fc40")),
        (ARCH_TAG, RawValue::Str("x86_64")),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.version.raw, "1.2.3-2.fc40");
}

#[test]
fn epoch_prefix_is_parsed_out_of_the_version() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("2:1.2.3")),
        (ARCH_TAG, RawValue::Str("x86_64")),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.version.epoch, 2);
    assert_eq!(decoded.meta.version.raw, "1.2.3");
}

#[test]
fn parses_all_optional_fields() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (RELEASE_TAG, RawValue::Str("1")),
        (ARCH_TAG, RawValue::Str("x86_64")),
        (SUMMARY_TAG, RawValue::Str("A test package")),
        (LICENSE_TAG, RawValue::Str("MIT")),
        (PACKAGER_TAG, RawValue::Str("Jane <jane@example.com>")),
        (URL_TAG, RawValue::Str("https://example.com")),
        (SIZE_TAG, RawValue::I32(4096)),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.description, "A test package");
    assert_eq!(decoded.meta.license, Some("MIT".to_owned()));
    assert_eq!(decoded.meta.maintainer, "Jane <jane@example.com>");
    assert_eq!(decoded.meta.url, Some("https://example.com".to_owned()));
    assert_eq!(decoded.meta.installed_size, 4096);
}

#[test]
fn missing_name_is_malformed() {
    let header = build_header(&[
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (ARCH_TAG, RawValue::Str("x86_64")),
    ]);

    let result = header.decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn missing_version_is_malformed() {
    let header = build_header(&[(NAME_TAG, RawValue::Str("foo")), (ARCH_TAG, RawValue::Str("x86_64"))]);

    let result = header.decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn missing_arch_is_malformed() {
    let header = build_header(&[(NAME_TAG, RawValue::Str("foo")), (VERSION_TAG, RawValue::Str("1.2.3"))]);

    let result = header.decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn parses_dependencies_with_all_constraint_operators() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (ARCH_TAG, RawValue::Str("x86_64")),
        (
            REQUIRE_NAME_TAG,
            RawValue::StrArray(&["libc", "glibc", "bash", "coreutils", "sh"]),
        ),
        (
            REQUIRE_VERSION_TAG,
            RawValue::StrArray(&["2.34", "2.34", "5.0", "", ""]),
        ),
        (REQUIRE_FLAGS_TAG, RawValue::I32Array(&[0x02, 0x0A, 0x04, 0x00, 0x08])),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.dependencies.len(), 5);
    assert_eq!(decoded.dependencies[0].constraint, CONSTRAINT_LESS);
    assert_eq!(decoded.dependencies[1].constraint, CONSTRAINT_LESS | CONSTRAINT_EQUAL);
    assert_eq!(decoded.dependencies[2].constraint, CONSTRAINT_GREATER);
    assert_eq!(decoded.dependencies[3].constraint, CONSTRAINT_ANY);
    assert_eq!(decoded.dependencies[4].constraint, CONSTRAINT_EQUAL);
    assert_eq!(decoded.dependencies[0].version.raw, "2.34");
    assert_eq!(decoded.dependencies[3].name, "coreutils");
}

#[test]
fn filters_out_rpmlib_internal_dependencies() {
    let header = build_header(&[
        (NAME_TAG, RawValue::Str("foo")),
        (VERSION_TAG, RawValue::Str("1.2.3")),
        (ARCH_TAG, RawValue::Str("x86_64")),
        (REQUIRE_NAME_TAG, RawValue::StrArray(&["rpmlib(PayloadIsXz)", "bash"])),
        (REQUIRE_VERSION_TAG, RawValue::StrArray(&["4.14.3-1", ""])),
        (REQUIRE_FLAGS_TAG, RawValue::I32Array(&[0x0100_0000 | 0x08, 0x00])),
    ]);

    let decoded = header.decode(CHECKSUM).unwrap();

    assert_eq!(decoded.dependencies.len(), 1);
    assert_eq!(decoded.dependencies[0].name, "bash");
}
