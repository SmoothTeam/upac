// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_decoder_alpm::pkginfo::PkgInfo;

use upac_types::decoder::DecodeError;
use upac_types::package::{Version, VersionConstraint, VersionRequirement};
use upac_types::traits::DecodeMeta;

const CHECKSUM: [u8; 32] = [7; 32];

fn bounded(constraint: VersionConstraint, raw_version: &str) -> VersionRequirement {
    VersionRequirement::Bounded {
        constraint,
        version: Version::parse(raw_version),
    }
}

#[test]
fn parses_minimal_pkginfo_with_defaults() {
    let content = "pkgname = foo\npkgver = 1.2.3\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.name, "foo");
    assert_eq!(decoded.meta.version.raw, "1.2.3");
    assert_eq!(decoded.meta.version.epoch, 0);
    assert_eq!(decoded.meta.arch, "any");
    assert_eq!(decoded.meta.maintainer, "");
    assert_eq!(decoded.meta.description, "");
    assert_eq!(decoded.meta.license, None);
    assert_eq!(decoded.meta.url, None);
    assert_eq!(decoded.meta.sha256, CHECKSUM);
    assert!(decoded.dependencies.is_empty());
}

#[test]
fn combines_pkgver_and_pkgrel_into_the_raw_version() {
    let content = "pkgname = foo\npkgver = 1.2.3\npkgrel = 2\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.version.raw, "1.2.3-2");
}

#[test]
fn parses_epoch_separately_from_the_version() {
    let content = "pkgname = foo\npkgver = 1.2.3\nepoch = 2\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.version.epoch, 2);
    assert_eq!(decoded.meta.version.raw, "1.2.3");
}

#[test]
fn parses_all_fields_and_ignores_comments_and_blank_lines() {
    let content = "# a comment\n\npkgname = foo\npkgver = 1.2.3\narch = x86_64\npkgdesc = A test package\nurl = \
                   https://example.com\npackager = Jane <jane@example.com>\nlicense = MIT\nsize = 4096\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    assert_eq!(decoded.meta.arch, "x86_64");
    assert_eq!(decoded.meta.description, "A test package");
    assert_eq!(decoded.meta.url, Some("https://example.com".to_owned()));
    assert_eq!(decoded.meta.maintainer, "Jane <jane@example.com>");
    assert_eq!(decoded.meta.license, Some("MIT".to_owned()));
    assert_eq!(decoded.meta.installed_size, 4096);
}

#[test]
fn missing_pkgname_is_malformed() {
    let content = "pkgver = 1.2.3\n";

    let result = PkgInfo(content).decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn missing_pkgver_is_malformed() {
    let content = "pkgname = foo\n";

    let result = PkgInfo(content).decode(CHECKSUM);

    assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
}

#[test]
fn parses_dependencies_with_every_constraint_operator() {
    let content = "pkgname = foo\npkgver = 1.2.3\ndepend = bash\ndepend = glibc>=2.36\ndepend = openssl<=3\ndepend = \
                   python=3.12\ndepend = zlib<2\ndepend = curl>7\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    let dependencies = decoded.dependencies;
    assert_eq!(dependencies.len(), 6);

    assert_eq!(dependencies[0].name, "bash");
    assert_eq!(dependencies[0].requirement, VersionRequirement::Any);

    assert_eq!(dependencies[1].name, "glibc");
    assert_eq!(
        dependencies[1].requirement,
        bounded(VersionConstraint::GreaterOrEqual, "2.36")
    );

    assert_eq!(dependencies[2].name, "openssl");
    assert_eq!(
        dependencies[2].requirement,
        bounded(VersionConstraint::LessOrEqual, "3")
    );

    assert_eq!(dependencies[3].name, "python");
    assert_eq!(dependencies[3].requirement, bounded(VersionConstraint::Equal, "3.12"));

    assert_eq!(dependencies[4].name, "zlib");
    assert_eq!(dependencies[4].requirement, bounded(VersionConstraint::Less, "2"));

    assert_eq!(dependencies[5].name, "curl");
    assert_eq!(dependencies[5].requirement, bounded(VersionConstraint::Greater, "7"));
}

#[test]
fn parses_a_dependency_version_with_its_own_epoch() {
    let content = "pkgname = foo\npkgver = 1.2.3\ndepend = python>=2:3.10\n";

    let decoded = PkgInfo(content).decode(CHECKSUM).unwrap();

    assert_eq!(
        decoded.dependencies[0].requirement,
        VersionRequirement::Bounded {
            constraint: VersionConstraint::GreaterOrEqual,
            version: Version {
                epoch: 2,
                raw: "3.10".to_owned(),
            },
        }
    );
}

#[test]
fn a_dependency_operator_without_a_version_is_malformed() {
    for content in [
        "pkgname = foo\npkgver = 1.2.3\ndepend = glibc>=\n",
        "pkgname = foo\npkgver = 1.2.3\ndepend = glibc>=2:\n",
    ] {
        let result = PkgInfo(content).decode(CHECKSUM);

        assert_eq!(result.unwrap_err(), DecodeError::MalformedMetadata);
    }
}
