// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use super::deb::{POSTINST_FILE, POSTRM_FILE, PREINST_FILE, PRERM_FILE};

pub fn scan(scripts_present: &[String]) -> Vec<PackageTrigger> {
    TriggerPosition::ALL
        .into_iter()
        .filter_map(|position| {
            let name = native_name(position);

            scripts_present
                .iter()
                .any(|script| script == name)
                .then(|| PackageTrigger {
                    position,
                    name: name.to_owned(),
                })
        })
        .collect()
}

fn native_name(position: TriggerPosition) -> &'static str {
    match position {
        TriggerPosition::PreInstall | TriggerPosition::PreUpgrade => PREINST_FILE,
        TriggerPosition::PostInstall | TriggerPosition::PostUpgrade => POSTINST_FILE,
        TriggerPosition::PreRemove => PRERM_FILE,
        TriggerPosition::PostRemove => POSTRM_FILE,
    }
}
