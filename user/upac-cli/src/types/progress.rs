// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::os::raw::c_void;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use upac_abi::error::ErrorDomain;
use upac_abi::hook::{CProgressEvent, HookAck};

use crate::types::errors::StageName;

const SPINNER_TEMPLATE: &str = "{spinner:.cyan} {msg}";
const BAR_TEMPLATE: &str = "{spinner:.cyan} [{bar:32.cyan/blue}] {pos}/{len} {msg}";
const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct ProgressState {
    bar: ProgressBar,
    domain: ErrorDomain,
    is_bar: bool,
}

impl ProgressState {
    pub fn new(domain: ErrorDomain) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(spinner_style());
        bar.enable_steady_tick(TICK_INTERVAL);

        ProgressState {
            bar,
            domain,
            is_bar: false,
        }
    }

    pub fn ctx_ptr(&mut self) -> *mut c_void {
        std::ptr::from_mut(self).cast()
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }

    fn apply(&mut self, event: &CProgressEvent) {
        let stage = StageName::new(self.domain, event.stage).to_string();
        let subject = <&str>::try_from(&event.subject).unwrap_or_default();

        if event.total > 0 {
            if !self.is_bar {
                self.bar.set_style(bar_style());
                self.is_bar = true;
            }
            self.bar.set_length(event.total);
            self.bar.set_position(event.current);
        }

        let message = if subject.is_empty() {
            stage
        } else {
            format!("{stage}: {subject}")
        };
        self.bar.set_message(message);
    }
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template(SPINNER_TEMPLATE).unwrap_or_else(|_| ProgressStyle::default_spinner())
}

fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template(BAR_TEMPLATE).unwrap_or_else(|_| ProgressStyle::default_bar())
}

/// # Safety
/// `ctx` must be a valid, live pointer to a `ProgressState` for the whole duration of the FFI
/// call that this hook is registered for (guaranteed by construction: callers keep their
/// `ProgressState` on the stack for exactly that call, calling `ctx_ptr()` only after it's in
/// its final resting place).
pub unsafe extern "C" fn on_progress(event: *const CProgressEvent, ctx: *mut c_void) -> HookAck {
    let state = unsafe { &mut *ctx.cast::<ProgressState>() };
    let event = unsafe { &*event };

    state.apply(event);

    HookAck::Delivered
}
