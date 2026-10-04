// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use super::xbps::{INSTALL_ENTRY, POST_ACTION, PRE_ACTION, REMOVE_ENTRY};

pub fn scan(install_present: bool, remove_present: bool) -> Vec<PackageTrigger> {
    TriggerPosition::ALL
        .into_iter()
        .filter_map(|position| {
            let (entry, action) = native(position);
            let present = if entry == INSTALL_ENTRY {
                install_present
            } else {
                remove_present
            };

            present.then(|| PackageTrigger {
                position,
                name: format!("{entry} {action}"),
            })
        })
        .collect()
}

fn native(position: TriggerPosition) -> (&'static str, &'static str) {
    match position {
        TriggerPosition::PreInstall | TriggerPosition::PreUpgrade => (INSTALL_ENTRY, PRE_ACTION),
        TriggerPosition::PostInstall | TriggerPosition::PostUpgrade => (INSTALL_ENTRY, POST_ACTION),
        TriggerPosition::PreRemove => (REMOVE_ENTRY, PRE_ACTION),
        TriggerPosition::PostRemove => (REMOVE_ENTRY, POST_ACTION),
    }
}
