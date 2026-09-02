// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null;

use upac_abi::package::{CPackageMeta, CVersion};
use upac_abi::types::CSlice;

use crate::locale;

use super::{PackageField, PackageFormatter, SizeDisplay, VersionDisplay};

fn slice_from_cstr(value: &CString) -> CSlice {
    CSlice {
        ptr: value.as_ptr().cast(),
        len: value.as_bytes().len(),
    }
}

fn optional_slice(value: Option<&CString>) -> CSlice {
    match value {
        Some(value) => slice_from_cstr(value),
        None => CSlice { ptr: null(), len: 0 },
    }
}

struct MetaFixture {
    name: CString,
    version_raw: CString,
    arch: CString,
    arch_sub: Option<CString>,
    maintainer: CString,
    description: CString,
    license: Option<CString>,
    url: Option<CString>,
}

impl MetaFixture {
    fn new(name: &str) -> Self {
        MetaFixture {
            name: CString::new(name).unwrap(),
            version_raw: CString::new("1.0.0").unwrap(),
            arch: CString::new("x86_64").unwrap(),
            arch_sub: None,
            maintainer: CString::new("someone").unwrap(),
            description: CString::new("a package").unwrap(),
            license: None,
            url: None,
        }
    }

    fn meta(&self, epoch: u32, installed_size: u64) -> CPackageMeta {
        CPackageMeta {
            struct_size: size_of::<CPackageMeta>(),
            name: slice_from_cstr(&self.name),
            version: CVersion {
                struct_size: size_of::<CVersion>(),
                epoch,
                raw: slice_from_cstr(&self.version_raw),
            },
            arch: slice_from_cstr(&self.arch),
            arch_sub: optional_slice(self.arch_sub.as_ref()),
            maintainer: slice_from_cstr(&self.maintainer),
            description: slice_from_cstr(&self.description),
            license: optional_slice(self.license.as_ref()),
            url: optional_slice(self.url.as_ref()),
            sha256: [0u8; 32],
            installed_size,
        }
    }
}

#[test]
fn version_display_omits_a_zero_epoch() {
    let raw = CString::new("1.2.3").unwrap();
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: slice_from_cstr(&raw),
    };

    assert_eq!(VersionDisplay(&version).to_string(), "1.2.3");
}

#[test]
fn version_display_prefixes_a_nonzero_epoch() {
    let raw = CString::new("1.2.3").unwrap();
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 2,
        raw: slice_from_cstr(&raw),
    };

    assert_eq!(VersionDisplay(&version).to_string(), "2:1.2.3");
}

#[test]
fn size_display_picks_the_right_unit() {
    assert_eq!(SizeDisplay(512).to_string(), "512 B");
    assert_eq!(SizeDisplay(2048).to_string(), "2 KB");
    assert_eq!(SizeDisplay(5 * 1024 * 1024).to_string(), "5.0 MB");
    assert_eq!(SizeDisplay(3 * 1024 * 1024 * 1024).to_string(), "3.0 GB");
}

#[test]
fn package_field_display_resolves_the_localized_field_name() {
    locale::init_for_test();

    assert_eq!(PackageField::Architecture.display(), "architecture");
    assert_eq!(PackageField::Description.display(), "Description");
}

#[test]
fn field_value_formats_architecture_with_and_without_a_sub_arch() {
    let mut fixture = MetaFixture::new("upac");
    let meta = fixture.meta(0, 100);
    assert_eq!(
        PackageFormatter::field_value(&meta, PackageField::Architecture),
        "x86_64"
    );

    fixture.arch_sub = Some(CString::new("v3").unwrap());
    let meta = fixture.meta(0, 100);
    assert_eq!(
        PackageFormatter::field_value(&meta, PackageField::Architecture),
        "x86_64/v3"
    );
}

#[test]
fn field_value_formats_optional_fields_as_empty_when_absent() {
    let fixture = MetaFixture::new("upac");
    let meta = fixture.meta(0, 100);

    assert_eq!(PackageFormatter::field_value(&meta, PackageField::License), "");
    assert_eq!(PackageFormatter::field_value(&meta, PackageField::Url), "");
}

#[test]
fn field_value_formats_the_checksum_as_hex() {
    let fixture = MetaFixture::new("upac");
    let mut meta = fixture.meta(0, 100);
    meta.sha256 = [0xAB; 32];

    assert_eq!(
        PackageFormatter::field_value(&meta, PackageField::Checksum),
        "ab".repeat(32)
    );
}

#[test]
fn field_value_formats_size_and_version() {
    let fixture = MetaFixture::new("upac");
    let meta = fixture.meta(0, 2048);

    assert_eq!(PackageFormatter::field_value(&meta, PackageField::Size), "2 KB");
    assert_eq!(PackageFormatter::field_value(&meta, PackageField::Version), "1.0.0");
}

#[test]
fn ordered_metas_sorts_by_version_when_requested() {
    let older = MetaFixture::new("a");
    let mut newer = MetaFixture::new("b");
    newer.version_raw = CString::new("2.0.0").unwrap();

    let metas = [newer.meta(0, 1), older.meta(0, 1)];
    let formatter = PackageFormatter {
        extra_fields: &[],
        metas: &metas,
        sort: Some(PackageField::Version),
    };

    let ordered = formatter.ordered_metas();
    assert_eq!(<&str>::try_from(&ordered[0].name).unwrap(), "a");
    assert_eq!(<&str>::try_from(&ordered[1].name).unwrap(), "b");
}

#[test]
fn ordered_metas_sorts_by_size_when_requested() {
    let small = MetaFixture::new("small");
    let large = MetaFixture::new("large");

    let metas = [large.meta(0, 1000), small.meta(0, 10)];
    let formatter = PackageFormatter {
        extra_fields: &[],
        metas: &metas,
        sort: Some(PackageField::Size),
    };

    let ordered = formatter.ordered_metas();
    assert_eq!(<&str>::try_from(&ordered[0].name).unwrap(), "small");
    assert_eq!(<&str>::try_from(&ordered[1].name).unwrap(), "large");
}
