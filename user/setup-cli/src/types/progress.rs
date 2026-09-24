// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::os::raw::c_void;
use std::ptr::from_mut;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use upac_abi::error::ErrorDomain;
use upac_abi::hook::{CProgressEvent, HookAck};

use crate::layout::progress;
use crate::types::errors::StageName;

#[cfg(test)]
#[path = "../../tests/inline/progress.rs"]
mod tests;

/// # Safety
/// `ctx` must be a valid, live pointer to a `ProgressState` for the whole duration of the call
/// that this hook is registered for.
pub unsafe extern "C" fn on_progress(event: *const CProgressEvent, ctx: *mut c_void) -> HookAck {
    let state = unsafe { &mut *ctx.cast::<ProgressState>() };
    let event = unsafe { &*event };

    state.apply(event);

    HookAck::Delivered
}

pub struct ProgressState {
    pub(crate) bar: ProgressBar,
    pub(crate) is_bar: bool,

    domain: ErrorDomain,
}

impl ProgressState {
    pub fn new(domain: ErrorDomain) -> Self {
        let bar = ProgressBar::new_spinner();

        bar.set_style(Self::spinner_style());
        bar.enable_steady_tick(Duration::from_millis(u64::from(progress::TICK_INTERVAL_MS)));

        ProgressState {
            bar,
            is_bar: false,
            domain,
        }
    }

    pub fn ctx_ptr(&mut self) -> *mut c_void {
        from_mut(self).cast()
    }

    pub(crate) fn apply(&mut self, event: &CProgressEvent) {
        let stage = StageName::new(self.domain, event.stage).to_string();
        let subject = <&str>::try_from(&event.subject).unwrap_or_default();

        if event.total > 0 {
            if !self.is_bar {
                self.bar.set_style(Self::bar_style());
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

impl ProgressState {
    fn spinner_style() -> ProgressStyle {
        ProgressStyle::with_template(progress::SPINNER_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_chars(progress::TICK_CHARS)
    }

    fn bar_style() -> ProgressStyle {
        ProgressStyle::with_template(progress::BAR_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .tick_chars(progress::TICK_CHARS)
    }
}

impl Drop for ProgressState {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}
