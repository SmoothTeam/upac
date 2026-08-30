// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::DecoderTrigger;

use crate::deb::{POSTINST_FILE, POSTRM_FILE, PREINST_FILE, PRERM_FILE};

pub fn scan(scripts_present: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for trigger in DecoderTrigger::ALL {
        let name = native_name(trigger);
        let declared = scripts_present.iter().any(|script| script == name);
        let already_added = names.iter().any(|existing| existing == name);

        if declared && !already_added {
            names.push(name.to_owned());
        }
    }

    names
}

fn native_name(trigger: DecoderTrigger) -> &'static str {
    match trigger {
        DecoderTrigger::PreInstall | DecoderTrigger::PreUpgrade => PREINST_FILE,
        DecoderTrigger::PostInstall | DecoderTrigger::PostUpgrade => POSTINST_FILE,
        DecoderTrigger::PreRemove => PRERM_FILE,
        DecoderTrigger::PostRemove => POSTRM_FILE,
    }
}
