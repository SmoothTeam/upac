// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicUsize, Ordering};

use upac_abi::hook::{CProgressEvent, HookAck};

use upac_orchestrator::context::Context;
use upac_orchestrator::progress::{Message, ProgressEventBuilder};

static ALWAYS_RETRYING_CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn always_retrying_hook(_event: *const CProgressEvent, _ctx: *mut c_void) -> u8 {
    ALWAYS_RETRYING_CALLS.fetch_add(1, Ordering::SeqCst);

    HookAck::Retry.into()
}

unsafe extern "C" fn retrying_hook(_event: *const CProgressEvent, _ctx: *mut c_void) -> u8 {
    HookAck::Retry.into()
}

unsafe extern "C" fn garbage_hook(_event: *const CProgressEvent, _ctx: *mut c_void) -> u8 {
    u8::MAX
}

#[test]
fn a_missing_hook_counts_as_delivered() {
    let message = Message::new(None, null_mut());

    assert_eq!(message.send(&ProgressEventBuilder::new(0).build()), HookAck::Delivered);
}

#[test]
fn a_known_ack_is_passed_through() {
    let message = Message::new(Some(retrying_hook), null_mut());

    assert_eq!(message.send(&ProgressEventBuilder::new(0).build()), HookAck::Retry);
}

#[test]
fn an_unknown_ack_counts_as_delivered() {
    let message = Message::new(Some(garbage_hook), null_mut());

    assert_eq!(message.send(&ProgressEventBuilder::new(0).build()), HookAck::Delivered);
}

#[test]
fn send_progress_gives_up_on_a_hook_that_always_retries() {
    let mut context = Context::default();
    context.put(Message::new(Some(always_retrying_hook), null_mut()));

    context.send_progress(&ProgressEventBuilder::new(0));

    assert!(ALWAYS_RETRYING_CALLS.load(Ordering::SeqCst) > 1);
}
