// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use upac_decoder_xbps::triggers;

fn positions(triggers: &[PackageTrigger]) -> Vec<(TriggerPosition, &str)> {
    triggers
        .iter()
        .map(|trigger| (trigger.position, trigger.name.as_str()))
        .collect()
}

#[test]
fn finds_no_triggers_when_neither_script_is_present() {
    let triggers = triggers::scan(false, false);

    assert!(triggers.is_empty());
}

#[test]
fn the_install_script_covers_install_and_upgrade_before_and_after() {
    let triggers = triggers::scan(true, false);

    assert_eq!(
        positions(&triggers),
        vec![
            (TriggerPosition::PreInstall, "INSTALL pre"),
            (TriggerPosition::PostInstall, "INSTALL post"),
            (TriggerPosition::PreUpgrade, "INSTALL pre"),
            (TriggerPosition::PostUpgrade, "INSTALL post"),
        ]
    );
}

#[test]
fn the_remove_script_covers_both_remove_positions() {
    let triggers = triggers::scan(false, true);

    assert_eq!(
        positions(&triggers),
        vec![
            (TriggerPosition::PreRemove, "REMOVE pre"),
            (TriggerPosition::PostRemove, "REMOVE post")
        ]
    );
}

#[test]
fn finds_both_scripts_when_both_are_present() {
    let triggers = triggers::scan(true, true);

    assert_eq!(triggers.len(), 6);
}
