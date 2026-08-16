// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::size_of;
use std::ptr::null;

use upac_abi::decoder::{CDependency, CTriggerEntry, CTriggerTable};
use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageInfo, CPackageMeta, CVersion};
use upac_abi::types::{COwned, CSlice, CVec};

fn valid_version() -> CVersion {
    CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        release: 1,
        parts: CVec::from_owned(vec![1, 0, 0]),
        pre: CSlice { ptr: null(), len: 0 },
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
fn version_validate_rejects_empty_parts() {
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        release: 1,
        parts: CVec::from_owned(Vec::new()),
        pre: CSlice { ptr: null(), len: 0 },
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
    unsafe { upac_abi::memory::free_cslice(&info.arch) };
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
        upac_abi::memory::free_cslice(&dependency.name);
        dependency.version.free();
    }
}

#[test]
fn trigger_table_validate_ok_for_valid_entries() {
    let table = CTriggerTable {
        struct_size: size_of::<CTriggerTable>(),
        entries: CVec::from_owned(vec![
            CTriggerEntry {
                struct_size: size_of::<CTriggerEntry>(),
                name: CSlice::from_owned(b"pre-install".to_vec()),
                hook_id: 0,
            },
            CTriggerEntry {
                struct_size: size_of::<CTriggerEntry>(),
                name: CSlice::from_owned(b"post-install".to_vec()),
                hook_id: 1,
            },
        ]),
    };

    assert!(unsafe { table.validate() }.is_ok());
    unsafe {
        for entry in table.entries.as_slice() {
            upac_abi::memory::free_cslice(&entry.name);
        }
        upac_abi::memory::free_cvec(&table.entries);
    }
}

#[test]
fn trigger_table_validate_rejects_malformed_entry() {
    let table = CTriggerTable {
        struct_size: size_of::<CTriggerTable>(),
        entries: CVec::from_owned(vec![CTriggerEntry {
            struct_size: 0,
            name: CSlice::from_owned(b"pre-install".to_vec()),
            hook_id: 0,
        }]),
    };

    assert_eq!(unsafe { table.validate() }, Err(ErrorKind::AbiMismatch));
    unsafe {
        for entry in table.entries.as_slice() {
            upac_abi::memory::free_cslice(&entry.name);
        }
        upac_abi::memory::free_cvec(&table.entries);
    }
}
