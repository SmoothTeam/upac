// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::decoder::{CONSTRAINT_ANY, CONSTRAINT_EQUAL, CONSTRAINT_GREATER, CONSTRAINT_LESS};

use upac_decoder_deb::control::ControlFile;
use upac_decoder_deb::error::DecodeError;

const CHECKSUM: [u8; 32] = [7; 32];

#[test]
fn parses_minimal_control_with_defaults() {
    let content = "Package: foo\nVersion: 1.2.3\n";

    let control = ControlFile::parse(content, None, CHECKSUM).unwrap();

    assert_eq!(control.meta.name, "foo");
    assert_eq!(control.meta.version.raw, "1.2.3");
    assert_eq!(control.meta.version.epoch, 0);
    assert_eq!(control.meta.arch, "all");
    assert_eq!(control.meta.maintainer, "");
    assert_eq!(control.meta.description, "");
    assert_eq!(control.meta.license, None);
    assert_eq!(control.meta.url, None);
    assert_eq!(control.meta.sha256, CHECKSUM);
    assert!(control.dependencies.is_empty());
}

#[test]
fn parses_epoch_out_of_the_version_string() {
    let content = "Package: foo\nVersion: 2:1.2.3-1\n";

    let control = ControlFile::parse(content, None, CHECKSUM).unwrap();

    assert_eq!(control.meta.version.epoch, 2);
    assert_eq!(control.meta.version.raw, "1.2.3-1");
}

#[test]
fn parses_all_fields_and_ignores_blank_lines() {
    let content = "Package: foo\n\nVersion: 1.2.3\nArchitecture: amd64\nDescription: A test package\nHomepage: \
                   https://example.com\nMaintainer: Jane <jane@example.com>\nInstalled-Size: 4096\n";

    let control = ControlFile::parse(content, Some("MIT".to_owned()), CHECKSUM).unwrap();

    assert_eq!(control.meta.arch, "amd64");
    assert_eq!(control.meta.description, "A test package");
    assert_eq!(control.meta.url, Some("https://example.com".to_owned()));
    assert_eq!(control.meta.maintainer, "Jane <jane@example.com>");
    assert_eq!(control.meta.license, Some("MIT".to_owned()));
    assert_eq!(control.meta.installed_size, 4096);
}

#[test]
fn missing_package_is_malformed() {
    let content = "Version: 1.2.3\n";

    let result = ControlFile::parse(content, None, CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedControl);
}

#[test]
fn missing_version_is_malformed() {
    let content = "Package: foo\n";

    let result = ControlFile::parse(content, None, CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedControl);
}

#[test]
fn parses_dependencies_with_every_constraint_operator() {
    let content = "Package: foo\nVersion: 1.2.3\nDepends: bash, libc6 (>= 2.36), libssl (<= 3), libfoo (= 1.0), \
                   zlib1g (<< 2), libbar (>> 7)\n";

    let control = ControlFile::parse(content, None, CHECKSUM).unwrap();

    let dependencies = control.dependencies;
    assert_eq!(dependencies.len(), 6);

    assert_eq!(dependencies[0].name, "bash");
    assert_eq!(dependencies[0].constraint, CONSTRAINT_ANY);

    assert_eq!(dependencies[1].name, "libc6");
    assert_eq!(dependencies[1].constraint, CONSTRAINT_GREATER | CONSTRAINT_EQUAL);
    assert_eq!(dependencies[1].version.raw, "2.36");

    assert_eq!(dependencies[2].name, "libssl");
    assert_eq!(dependencies[2].constraint, CONSTRAINT_LESS | CONSTRAINT_EQUAL);
    assert_eq!(dependencies[2].version.raw, "3");

    assert_eq!(dependencies[3].name, "libfoo");
    assert_eq!(dependencies[3].constraint, CONSTRAINT_EQUAL);
    assert_eq!(dependencies[3].version.raw, "1.0");

    assert_eq!(dependencies[4].name, "zlib1g");
    assert_eq!(dependencies[4].constraint, CONSTRAINT_LESS);
    assert_eq!(dependencies[4].version.raw, "2");

    assert_eq!(dependencies[5].name, "libbar");
    assert_eq!(dependencies[5].constraint, CONSTRAINT_GREATER);
    assert_eq!(dependencies[5].version.raw, "7");
}

#[test]
fn picks_the_first_alternative_in_an_or_group() {
    let content = "Package: foo\nVersion: 1.2.3\nDepends: libfoo | libbar (>= 2.0)\n";

    let control = ControlFile::parse(content, None, CHECKSUM).unwrap();

    assert_eq!(control.dependencies.len(), 1);
    assert_eq!(control.dependencies[0].name, "libfoo");
    assert_eq!(control.dependencies[0].constraint, CONSTRAINT_ANY);
}
