// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;
use std::mem::size_of;

use upac_abi::error::ErrorDomain;
use upac_abi::hook::CProgressEvent;
use upac_abi::types::CSlice;

use upac_types::state::setup::{BootstrapStateId, PartitionAddStateId};

use crate::locale;

use super::ProgressState;

fn empty_slice() -> CSlice {
    CSlice {
        ptr: std::ptr::null(),
        len: 0,
    }
}

fn slice_from_cstr(value: &CString) -> CSlice {
    CSlice {
        ptr: value.as_ptr().cast(),
        len: value.as_bytes().len(),
    }
}

fn event(stage: u32, current: u64, total: u64, subject: CSlice) -> CProgressEvent {
    CProgressEvent {
        struct_size: size_of::<CProgressEvent>(),
        stage,
        phase: 0,
        subject,
        current,
        total,
    }
}

#[test]
fn apply_with_zero_total_stays_on_spinner() {
    locale::init_for_test();
    let mut state = ProgressState::new(ErrorDomain::Bootstrap);

    state.apply(&event(BootstrapStateId::EnumeratePackages as u32, 0, 0, empty_slice()));

    assert!(!state.is_bar);
    assert_eq!(state.bar.message(), "Enumerating packages");
}

#[test]
fn apply_with_nonzero_total_switches_to_bar_and_sets_position() {
    locale::init_for_test();
    let mut state = ProgressState::new(ErrorDomain::Bootstrap);

    state.apply(&event(BootstrapStateId::ImportPackage as u32, 3, 10, empty_slice()));

    assert!(state.is_bar);
    assert_eq!(state.bar.length(), Some(10));
    assert_eq!(state.bar.position(), 3);
}

#[test]
fn apply_includes_subject_in_message_when_present() {
    locale::init_for_test();
    let mut state = ProgressState::new(ErrorDomain::Bootstrap);
    let subject = CString::new("foo.txt").unwrap();

    state.apply(&event(
        BootstrapStateId::EnumeratePackages as u32,
        0,
        0,
        slice_from_cstr(&subject),
    ));

    assert_eq!(state.bar.message(), "Enumerating packages: foo.txt");
}

#[test]
fn apply_resolves_the_localized_stage_key() {
    locale::init_for_test();
    let mut state = ProgressState::new(ErrorDomain::Bootstrap);

    state.apply(&event(BootstrapStateId::StageBoot as u32, 0, 0, empty_slice()));

    assert_eq!(state.bar.message(), "Staging boot entry");
}

#[test]
fn apply_resolves_the_stage_through_the_domain_it_was_created_for() {
    locale::init_for_test();
    let mut state = ProgressState::new(ErrorDomain::PartitionAdd);

    state.apply(&event(PartitionAddStateId::InsertEntry as u32, 0, 0, empty_slice()));

    assert_eq!(state.bar.message(), "Adding partition");
}
