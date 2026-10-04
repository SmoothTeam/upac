// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::size_of;
use std::ptr::{null, null_mut};
use std::sync::atomic::AtomicU8;

use upac_abi::CONSTRAINT_ANY;
use upac_abi::error::AbiError;
use upac_abi::memory::free_cslice;
use upac_abi::package::{CPackageDependency, CPackageInfo, CPackageMeta, CVersion};
use upac_abi::request::CRequestBase;
use upac_abi::response::entry::{
    CConfigCommitEntry, CDiffConfigFileEntry, CDiffFileCommonEntry, CDiffPrefixFileEntry, CDiffUntrackedFileEntry,
    CHistoryEntry, CPrefixEntry, CSearchFileEntry,
};
use upac_abi::types::{COwned, CSlice, CVec};

fn valid_version() -> CVersion {
    CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice::from_owned(b"1.0.0".to_vec()),
    }
}

fn absent_version() -> CVersion {
    CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice { ptr: null(), len: 0 },
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

    assert_eq!(unsafe { version.validate() }, Err(AbiError::AbiMismatch));
    unsafe { version.free() };
}

#[test]
fn version_validate_rejects_empty_raw() {
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice { ptr: null(), len: 0 },
    };

    assert_eq!(unsafe { version.validate() }, Err(AbiError::InvalidEntry));
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

    assert_eq!(unsafe { meta.validate() }, Err(AbiError::AbiMismatch));
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

    assert_eq!(unsafe { info.validate() }, Err(AbiError::InvalidEntry));
    unsafe { free_cslice(&info.arch) };
}

#[test]
fn dependency_validate_rejects_invalid_nested_version() {
    let mut dependency = CPackageDependency {
        struct_size: size_of::<CPackageDependency>(),
        name: CSlice::from_owned(b"glibc".to_vec()),
        constraint: 0b010,
        version: valid_version(),
    };
    dependency.version.struct_size = 0;

    assert_eq!(unsafe { dependency.validate() }, Err(AbiError::AbiMismatch));
    unsafe {
        free_cslice(&dependency.name);
        dependency.version.free();
    }
}

#[test]
fn version_validate_rejects_a_non_null_empty_raw() {
    let version = CVersion {
        struct_size: size_of::<CVersion>(),
        epoch: 0,
        raw: CSlice::from_owned(Vec::new()),
    };

    assert_eq!(unsafe { version.validate() }, Err(AbiError::InvalidEntry));
    unsafe { version.free() };
}

#[test]
fn dependency_validate_accepts_any_version_without_one() {
    let dependency = CPackageDependency {
        struct_size: size_of::<CPackageDependency>(),
        name: CSlice::from_owned(b"glibc".to_vec()),
        constraint: CONSTRAINT_ANY,
        version: absent_version(),
    };

    assert_eq!(unsafe { dependency.validate() }, Ok(()));
    unsafe { free_cslice(&dependency.name) };
}

#[test]
fn dependency_validate_requires_a_version_for_a_specific_constraint() {
    let dependency = CPackageDependency {
        struct_size: size_of::<CPackageDependency>(),
        name: CSlice::from_owned(b"glibc".to_vec()),
        constraint: 0b010,
        version: absent_version(),
    };

    assert_eq!(unsafe { dependency.validate() }, Err(AbiError::InvalidEntry));
    unsafe { free_cslice(&dependency.name) };
}

#[test]
fn dependency_validate_rejects_constraint_bits_outside_the_mask() {
    for constraint in [0, 0b1000] {
        let dependency = CPackageDependency {
            struct_size: size_of::<CPackageDependency>(),
            name: CSlice::from_owned(b"glibc".to_vec()),
            constraint,
            version: absent_version(),
        };

        assert_eq!(unsafe { dependency.validate() }, Err(AbiError::InvalidEntry));
        unsafe { free_cslice(&dependency.name) };
    }
}

fn valid_diff_common() -> CDiffFileCommonEntry {
    CDiffFileCommonEntry {
        struct_size: size_of::<CDiffFileCommonEntry>(),
        path: CSlice::from_owned(b"/etc/upac.conf".to_vec()),
        kind: 2,
    }
}

#[test]
fn diff_file_entry_common_validate_ok_for_well_formed() {
    let common = valid_diff_common();

    assert!(unsafe { common.validate() }.is_ok());
    unsafe { common.free() };
}

#[test]
fn diff_file_entry_common_validate_rejects_wrong_struct_size() {
    let mut common = valid_diff_common();
    common.struct_size = 0;

    assert_eq!(unsafe { common.validate() }, Err(AbiError::AbiMismatch));
    unsafe { common.free() };
}

#[test]
fn diff_file_entry_common_validate_rejects_empty_path() {
    let common = CDiffFileCommonEntry {
        struct_size: size_of::<CDiffFileCommonEntry>(),
        path: CSlice { ptr: null(), len: 0 },
        kind: 2,
    };

    assert_eq!(unsafe { common.validate() }, Err(AbiError::InvalidEntry));
}

fn valid_diff_prefix_file_entry() -> CDiffPrefixFileEntry {
    CDiffPrefixFileEntry {
        struct_size: size_of::<CDiffPrefixFileEntry>(),
        common: valid_diff_common(),
        source: 0,
        package_name: CSlice::from_owned(b"upac".to_vec()),
        is_user: false,
    }
}

#[test]
fn diff_prefix_file_entry_validate_ok_for_well_formed() {
    let entry = valid_diff_prefix_file_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn diff_prefix_file_entry_validate_rejects_invalid_nested_common() {
    let mut entry = valid_diff_prefix_file_entry();
    entry.common.struct_size = 0;

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::AbiMismatch));
    unsafe { entry.free() };
}

fn valid_diff_config_file_entry() -> CDiffConfigFileEntry {
    CDiffConfigFileEntry {
        struct_size: size_of::<CDiffConfigFileEntry>(),
        common: valid_diff_common(),
        package_name: CSlice { ptr: null(), len: 0 },
    }
}

#[test]
fn diff_config_file_entry_validate_ok_with_no_package_name() {
    let entry = valid_diff_config_file_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn diff_config_file_entry_validate_rejects_invalid_nested_common() {
    let mut entry = valid_diff_config_file_entry();
    entry.common.struct_size = 0;

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::AbiMismatch));
    unsafe { entry.free() };
}

fn valid_diff_untracked_file_entry() -> CDiffUntrackedFileEntry {
    CDiffUntrackedFileEntry {
        struct_size: size_of::<CDiffUntrackedFileEntry>(),
        common: valid_diff_common(),
        source: 1,
    }
}

#[test]
fn diff_untracked_file_entry_validate_ok_for_well_formed() {
    let entry = valid_diff_untracked_file_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn diff_untracked_file_entry_validate_rejects_invalid_nested_common() {
    let mut entry = valid_diff_untracked_file_entry();
    entry.common.struct_size = 0;

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::AbiMismatch));
    unsafe { entry.free() };
}

fn valid_config_commit_entry() -> CConfigCommitEntry {
    CConfigCommitEntry {
        struct_size: size_of::<CConfigCommitEntry>(),
        config_digest: CSlice::from_owned(b"deadbeef".to_vec()),
        subject: CSlice::from_owned(b"install".to_vec()),
        message: CSlice { ptr: null(), len: 0 },
    }
}

#[test]
fn config_commit_entry_validate_ok_for_well_formed() {
    let entry = valid_config_commit_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn config_commit_entry_validate_rejects_missing_config_digest() {
    let mut entry = valid_config_commit_entry();
    unsafe { free_cslice(&entry.config_digest) };
    entry.config_digest = CSlice { ptr: null(), len: 0 };

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::InvalidEntry));
    unsafe { entry.free() };
}

fn valid_search_file_entry() -> CSearchFileEntry {
    CSearchFileEntry {
        struct_size: size_of::<CSearchFileEntry>(),
        path: CSlice::from_owned(b"/etc/upac.conf".to_vec()),
        package_name: CSlice::from_owned(b"upac".to_vec()),
        is_user: false,
    }
}

#[test]
fn search_file_entry_validate_ok_for_well_formed() {
    let entry = valid_search_file_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn search_file_entry_validate_rejects_missing_path() {
    let mut entry = valid_search_file_entry();
    unsafe { free_cslice(&entry.path) };
    entry.path = CSlice { ptr: null(), len: 0 };

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::InvalidEntry));
    unsafe { entry.free() };
}

fn valid_prefix_entry() -> CPrefixEntry {
    CPrefixEntry {
        struct_size: size_of::<CPrefixEntry>(),
        prefix_digest: CSlice::from_owned(b"deadbeef".to_vec()),
        subject: CSlice::from_owned(b"install".to_vec()),
        message: CSlice { ptr: null(), len: 0 },
        timestamp: 0,
        working_config: CSlice { ptr: null(), len: 0 },
    }
}

#[test]
fn prefix_entry_validate_ok_for_well_formed() {
    let entry = valid_prefix_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn prefix_entry_validate_rejects_missing_prefix_digest() {
    let mut entry = valid_prefix_entry();
    unsafe { free_cslice(&entry.prefix_digest) };
    entry.prefix_digest = CSlice { ptr: null(), len: 0 };

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::InvalidEntry));
    unsafe { entry.free() };
}

fn valid_history_entry() -> CHistoryEntry {
    CHistoryEntry {
        struct_size: size_of::<CHistoryEntry>(),
        prefix_digest: CSlice::from_owned(b"deadbeef".to_vec()),
        subject: CSlice::from_owned(b"install".to_vec()),
        message: CSlice { ptr: null(), len: 0 },
        timestamp: 0,
        working_config: CSlice { ptr: null(), len: 0 },
        config_history: CVec {
            ptr: null_mut(),
            len: 0,
        },
    }
}

#[test]
fn history_entry_validate_ok_for_well_formed_with_empty_history() {
    let entry = valid_history_entry();

    assert!(unsafe { entry.validate() }.is_ok());
    unsafe { entry.free() };
}

#[test]
fn history_entry_validate_rejects_an_invalid_config_history_element() {
    let mut bad_commit = valid_config_commit_entry();
    bad_commit.struct_size = 0;
    let mut history = vec![bad_commit];

    let mut entry = valid_history_entry();
    entry.config_history = CVec {
        ptr: history.as_mut_ptr(),
        len: history.len(),
    };

    assert_eq!(unsafe { entry.validate() }, Err(AbiError::AbiMismatch));

    unsafe {
        history[0].free();
        entry.config_history = CVec {
            ptr: null_mut(),
            len: 0,
        };
        entry.free();
    }
}

fn valid_request_base(cancel_token: &AtomicU8) -> CRequestBase {
    CRequestBase {
        struct_size: size_of::<CRequestBase>(),
        on_hook: None,
        hook_ctx: null_mut(),
        cancel_token,
    }
}

#[test]
fn request_base_validate_ok_for_well_formed() {
    let cancel_token = AtomicU8::new(0);

    assert!(unsafe { valid_request_base(&cancel_token).validate() }.is_ok());
}

#[test]
fn request_base_validate_rejects_wrong_struct_size() {
    let cancel_token = AtomicU8::new(0);
    let mut base = valid_request_base(&cancel_token);
    base.struct_size = 0;

    assert_eq!(unsafe { base.validate() }, Err(AbiError::AbiMismatch));
}
