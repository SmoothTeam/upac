// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::{addr_of_mut, null_mut};

use upac_abi::hook::{CProgressEvent, CancelToken, HookAck, Message, MessageHook, ProgressEventBuilder};

unsafe extern "C" fn record_stage_and_retry(event: *const CProgressEvent, ctx: *mut c_void) -> HookAck {
    unsafe {
        *ctx.cast::<u32>() = (*event).stage;
    }
    HookAck::Retry
}

#[test]
fn cancel_token_starts_not_cancelled() {
    let token = CancelToken::new();

    assert!(!token.is_cancelled());
}

#[test]
fn cancel_token_default_starts_not_cancelled() {
    let token = CancelToken::default();

    assert!(!token.is_cancelled());
}

#[test]
fn cancel_token_cancel_is_observed() {
    let token = CancelToken::new();

    token.cancel();

    assert!(token.is_cancelled());
}

#[test]
fn cancel_token_reset_clears_a_cancellation() {
    let token = CancelToken::new();
    token.cancel();

    token.reset();

    assert!(!token.is_cancelled());
}

#[test]
fn progress_event_builder_defaults() {
    let event = ProgressEventBuilder::new(3).build();

    assert_eq!(event.struct_size, size_of::<CProgressEvent>());
    assert_eq!(event.stage, 3);
    assert_eq!(event.phase, 0);
    assert_eq!(event.current, 0);
    assert_eq!(event.total, 0);
    assert!(event.subject.ptr.is_null());
}

#[test]
fn progress_event_builder_stage_accessor_matches_the_constructor() {
    let builder = ProgressEventBuilder::new(7);

    assert_eq!(builder.stage(), 7);
}

#[test]
fn progress_event_builder_applies_phase_subject_and_progress() {
    let builder = ProgressEventBuilder::new(1).phase(2).subject("foo.txt").progress(3, 10);
    let event = builder.build();

    assert_eq!(event.phase, 2);
    assert_eq!(event.current, 3);
    assert_eq!(event.total, 10);
    assert_eq!(<&str>::try_from(&event.subject).unwrap(), "foo.txt");
}

#[test]
fn message_send_with_no_hook_returns_delivered() {
    let message = Message::new(None, null_mut());
    let event = ProgressEventBuilder::new(0).build();

    assert_eq!(message.send(&event), HookAck::Delivered);
}

#[test]
fn message_send_with_a_hook_forwards_the_event_and_context() {
    let mut recorded_stage: u32 = 0;
    let message = Message::new(Some(record_stage_and_retry), addr_of_mut!(recorded_stage).cast());
    let event = ProgressEventBuilder::new(9).build();

    let ack = message.send(&event);

    assert_eq!(ack, HookAck::Retry);
    assert_eq!(recorded_stage, 9);
}
