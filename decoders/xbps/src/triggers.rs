// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::DecoderTrigger;

use crate::xbps::{INSTALL_ENTRY, REMOVE_ENTRY};

pub fn scan(install_present: bool, remove_present: bool) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for trigger in DecoderTrigger::ALL {
        let name = native(trigger);
        let declared = if name == INSTALL_ENTRY {
            install_present
        } else {
            remove_present
        };
        let already_added = names.iter().any(|existing| existing == name);

        if declared && !already_added {
            names.push(name.to_owned());
        }
    }

    names
}

/// XBPS has no separate install-vs-upgrade or pre-vs-post script *files* — a single `INSTALL`
/// script is invoked with a `pre`/`post` argument for both fresh installs and upgrades, and a
/// single `REMOVE` script likewise covers both removal positions.
fn native(trigger: DecoderTrigger) -> &'static str {
    match trigger {
        DecoderTrigger::PreInstall
        | DecoderTrigger::PostInstall
        | DecoderTrigger::PreUpgrade
        | DecoderTrigger::PostUpgrade => INSTALL_ENTRY,
        DecoderTrigger::PreRemove | DecoderTrigger::PostRemove => REMOVE_ENTRY,
    }
}
