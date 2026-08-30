// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_decoder_xbps::triggers;

#[test]
fn finds_no_triggers_when_neither_script_is_present() {
    let triggers = triggers::scan(false, false);

    assert!(triggers.is_empty());
}

#[test]
fn a_bare_install_script_covers_both_install_and_upgrade_positions() {
    let triggers = triggers::scan(true, false);

    assert_eq!(triggers, vec!["INSTALL"]);
}

#[test]
fn a_bare_remove_script_covers_both_remove_positions() {
    let triggers = triggers::scan(false, true);

    assert_eq!(triggers, vec!["REMOVE"]);
}

#[test]
fn finds_both_scripts_when_both_are_present() {
    let mut triggers = triggers::scan(true, true);
    triggers.sort();

    assert_eq!(triggers, vec!["INSTALL", "REMOVE"]);
}
