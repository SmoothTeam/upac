// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use upac_decoder_deb::triggers;

fn positions(triggers: &[PackageTrigger]) -> Vec<(TriggerPosition, &str)> {
    triggers
        .iter()
        .map(|trigger| (trigger.position, trigger.name.as_str()))
        .collect()
}

#[test]
fn finds_no_triggers_when_no_scripts_are_present() {
    let triggers = triggers::scan(&[]);

    assert!(triggers.is_empty());
}

#[test]
fn a_bare_preinst_covers_both_install_and_upgrade_positions() {
    let scripts = vec!["preinst".to_owned()];

    let triggers = triggers::scan(&scripts);

    assert_eq!(
        positions(&triggers),
        vec![
            (TriggerPosition::PreInstall, "preinst"),
            (TriggerPosition::PreUpgrade, "preinst")
        ]
    );
}

#[test]
fn finds_all_four_maintainer_scripts() {
    let scripts = vec![
        "preinst".to_owned(),
        "postinst".to_owned(),
        "prerm".to_owned(),
        "postrm".to_owned(),
    ];

    let triggers = triggers::scan(&scripts);

    assert_eq!(
        positions(&triggers),
        vec![
            (TriggerPosition::PreInstall, "preinst"),
            (TriggerPosition::PostInstall, "postinst"),
            (TriggerPosition::PreUpgrade, "preinst"),
            (TriggerPosition::PostUpgrade, "postinst"),
            (TriggerPosition::PreRemove, "prerm"),
            (TriggerPosition::PostRemove, "postrm"),
        ]
    );
}

#[test]
fn ignores_names_that_are_not_declared_scripts() {
    let scripts = vec!["control".to_owned(), "md5sums".to_owned()];

    let triggers = triggers::scan(&scripts);

    assert!(triggers.is_empty());
}
