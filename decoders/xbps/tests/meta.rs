// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::decoder::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS, DecodeError};
use upac_decoder_xbps::meta::Props;
use upac_types::decoder::DecodeMeta;

const CHECKSUM: [u8; 32] = [7; 32];

fn plist(entries: &str) -> String {
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n{entries}</dict>\n</plist>\n")
}

fn escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn scalar(key: &str, value: &str) -> String {
    format!("<key>{key}</key>\n<string>{}</string>\n", escape(value))
}

fn integer(key: &str, value: u64) -> String {
    format!("<key>{key}</key>\n<integer>{value}</integer>\n")
}

fn array(key: &str, values: &[&str]) -> String {
    let items = values
        .iter()
        .map(|value| format!("<string>{}</string>\n", escape(value)))
        .collect::<String>();

    format!("<key>{key}</key>\n<array>\n{items}</array>\n")
}

#[test]
fn parses_minimal_meta_with_defaults() {
    let xml = plist(&(scalar("pkgname", "foo") + &scalar("version", "1.2.3")));

    let decoded = Props(&xml).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.name, "foo");
    assert_eq!(decoded.meta.version.raw, "1.2.3");
    assert_eq!(decoded.meta.version.epoch, 0);
    assert_eq!(decoded.meta.arch, "");
    assert_eq!(decoded.meta.maintainer, "");
    assert_eq!(decoded.meta.description, "");
    assert_eq!(decoded.meta.license, None);
    assert_eq!(decoded.meta.url, None);
    assert_eq!(decoded.meta.installed_size, 0);
    assert_eq!(decoded.meta.sha256, CHECKSUM);
    assert!(decoded.dependencies.is_empty());
}

#[test]
fn epoch_prefix_is_parsed_out_of_the_version() {
    let xml = plist(&(scalar("pkgname", "foo") + &scalar("version", "2:1.2.3")));

    let decoded = Props(&xml).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.version.epoch, 2);
    assert_eq!(decoded.meta.version.raw, "1.2.3");
}

#[test]
fn parses_all_optional_fields() {
    let xml = plist(
        &(scalar("pkgname", "foo")
            + &scalar("version", "1.2.3")
            + &scalar("architecture", "x86_64")
            + &scalar("short_desc", "A test package")
            + &scalar("license", "MIT")
            + &scalar("maintainer", "Jane <jane@example.com>")
            + &scalar("homepage", "https://example.com")
            + &integer("installed_size", 4096)),
    );

    let decoded = Props(&xml).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.arch, "x86_64");
    assert_eq!(decoded.meta.description, "A test package");
    assert_eq!(decoded.meta.license, Some("MIT".to_owned()));
    assert_eq!(decoded.meta.maintainer, "Jane <jane@example.com>");
    assert_eq!(decoded.meta.url, Some("https://example.com".to_owned()));
    assert_eq!(decoded.meta.installed_size, 4096);
}

#[test]
fn missing_pkgname_is_malformed() {
    let xml = plist(&scalar("version", "1.2.3"));

    let result = Props(&xml).decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn missing_version_is_malformed() {
    let xml = plist(&scalar("pkgname", "foo"));

    let result = Props(&xml).decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn parses_dependencies_with_all_constraint_operators() {
    let xml = plist(
        &(scalar("pkgname", "foo")
            + &scalar("version", "1.2.3")
            + &array("run_depends", &["bash", "glibc>=2.34", "libssl<=3", "coreutils"])),
    );

    let decoded = Props(&xml).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.dependencies.len(), 4);

    assert_eq!(decoded.dependencies[0].name, "bash");
    assert_eq!(decoded.dependencies[0].constraint, CONSTRAINT_ANY);

    assert_eq!(decoded.dependencies[1].name, "glibc");
    assert_eq!(
        decoded.dependencies[1].constraint,
        CONSTRAINT_GREATER | CONSTRAINT_EQUAL
    );
    assert_eq!(decoded.dependencies[1].version.raw, "2.34");

    assert_eq!(decoded.dependencies[2].name, "libssl");
    assert_eq!(decoded.dependencies[2].constraint, CONSTRAINT_LESS | CONSTRAINT_EQUAL);
    assert_eq!(decoded.dependencies[2].version.raw, "3");

    assert_eq!(decoded.dependencies[3].name, "coreutils");
    assert_eq!(decoded.dependencies[3].constraint, CONSTRAINT_ANY);
}
