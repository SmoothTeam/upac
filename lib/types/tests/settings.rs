// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::settings::{GcSettings, ProgressSettings, RuntimeSettings};

#[test]
fn gc_settings_default_retention_depth_is_five() {
    assert_eq!(GcSettings::default().retention_depth, 5);
}

#[test]
fn progress_settings_default_templates_and_tick_interval() {
    let settings = ProgressSettings::default();

    assert_eq!(settings.spinner_template, "{spinner:.cyan} {msg}");
    assert_eq!(
        settings.bar_template,
        "{spinner:.cyan} [{bar:32.cyan/blue}] {pos}/{len} {msg}"
    );
    assert_eq!(settings.tick_interval_ms, 100);
}

#[test]
fn runtime_settings_default_combines_gc_and_progress_defaults() {
    let settings = RuntimeSettings::default();

    assert_eq!(settings.gc.retention_depth, GcSettings::default().retention_depth);
    assert_eq!(
        settings.progress.tick_interval_ms,
        ProgressSettings::default().tick_interval_ms
    );
}
