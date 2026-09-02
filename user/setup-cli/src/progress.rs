// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::os::raw::c_void;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use upac_abi::hook::{CProgressEvent, HookAck};

use upac_setup::genesis::GenesisStage;

use upac_types::settings::{ProgressSettings, RuntimeSettings};

use crate::locale::LOADER;

#[cfg(test)]
#[path = "../tests/inline/progress.rs"]
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
    bar: ProgressBar,
    is_bar: bool,
    settings: ProgressSettings,
}

impl ProgressState {
    pub fn new() -> Self {
        let settings = RuntimeSettings::load().progress;

        let bar = ProgressBar::new_spinner();
        bar.set_style(Self::spinner_style(&settings.spinner_template));
        bar.enable_steady_tick(Duration::from_millis(settings.tick_interval_ms));

        ProgressState {
            bar,
            is_bar: false,
            settings,
        }
    }

    pub fn ctx_ptr(&mut self) -> *mut c_void {
        std::ptr::from_mut(self).cast()
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }

    fn apply(&mut self, event: &CProgressEvent) {
        let stage = Self::stage_name(event.stage);
        let subject = <&str>::try_from(&event.subject).unwrap_or_default();

        if event.total > 0 {
            if !self.is_bar {
                self.bar.set_style(Self::bar_style(&self.settings.bar_template));
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
    fn stage_name(index: u32) -> String {
        LOADER.get(GenesisStage::from_stage_index(index as usize).stage_key())
    }

    fn spinner_style(template: &str) -> ProgressStyle {
        ProgressStyle::with_template(template).unwrap_or_else(|_| ProgressStyle::default_spinner())
    }

    fn bar_style(template: &str) -> ProgressStyle {
        ProgressStyle::with_template(template).unwrap_or_else(|_| ProgressStyle::default_bar())
    }
}

impl Default for ProgressState {
    fn default() -> Self {
        Self::new()
    }
}
