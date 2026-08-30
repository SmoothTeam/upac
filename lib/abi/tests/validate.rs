// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::size_of;
use std::ptr::null;

use upac_abi::decoder::CDependency;
use upac_abi::error::ErrorKind;
use upac_abi::memory::free_cslice;
use upac_abi::package::{CPackageInfo, CPackageMeta, CVersion};
use upac_abi::types::{COwned, CSlice};

fn valid_version() -> CVersion {
    CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice::from_owned(b"1.0.0".to_vec()),
    }
}

fn valid_package_meta() -> CPackageMeta {
    CPackageMeta {
        struct_size: size_of::<CPackageMeta>(),
        name: CSlice::from_owned(b"upac".to_vec()),
        version: valid_version(),
        arch: CSlice::from_owned(b"x86_64".to_vec()),
        arch_sub: CSlice { ptr: null(), len: 0 },
        maintainer: CSlice::from_owned(b"JustPav".to_vec()),
        description: CSlice::from_owned(b"package manager".to_vec()),
        license: CSlice { ptr: null(), len: 0 },
        url: CSlice { ptr: null(), len: 0 },
        sha256: [0; 32],
        installed_size: 0,
    }
}

#[test]
fn version_validate_ok_for_well_formed() {
    let version = valid_version();

    assert!(unsafe { version.validate() }.is_ok());
    unsafe { version.free() };
}

#[test]
fn version_validate_rejects_wrong_struct_size() {
    let mut version = valid_version();
    version.struct_size = 0;

    assert_eq!(unsafe { version.validate() }, Err(ErrorKind::AbiMismatch));
    unsafe { version.free() };
}

#[test]
fn version_validate_rejects_empty_raw() {
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice { ptr: null(), len: 0 },
    };

    assert_eq!(unsafe { version.validate() }, Err(ErrorKind::InvalidEntry));
}

#[test]
fn package_meta_validate_ok_for_well_formed() {
    let meta = valid_package_meta();

    assert!(unsafe { meta.validate() }.is_ok());
    unsafe { meta.free() };
}

#[test]
fn package_meta_validate_rejects_invalid_nested_version() {
    let mut meta = valid_package_meta();
    meta.version.struct_size = 0;

    assert_eq!(unsafe { meta.validate() }, Err(ErrorKind::AbiMismatch));
    unsafe { meta.free() };
}

#[test]
fn package_info_validate_rejects_missing_required_field() {
    let info = CPackageInfo {
        struct_size: size_of::<CPackageInfo>(),
        name: CSlice { ptr: null(), len: 0 },
        arch: CSlice::from_owned(b"x86_64".to_vec()),
        arch_sub: CSlice { ptr: null(), len: 0 },
    };

    assert_eq!(unsafe { info.validate() }, Err(ErrorKind::InvalidEntry));
    unsafe { free_cslice(&info.arch) };
}

#[test]
fn dependency_validate_rejects_invalid_nested_version() {
    let mut dependency = CDependency {
        struct_size: size_of::<CDependency>(),
        name: CSlice::from_owned(b"glibc".to_vec()),
        constraint: 0b010,
        version: valid_version(),
    };
    dependency.version.struct_size = 0;

    assert_eq!(unsafe { dependency.validate() }, Err(ErrorKind::AbiMismatch));
    unsafe {
        free_cslice(&dependency.name);
        dependency.version.free();
    }
}
