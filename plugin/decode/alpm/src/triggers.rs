// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use super::layout::install::{POST_INSTALL, POST_REMOVE, POST_UPGRADE, PRE_INSTALL, PRE_REMOVE, PRE_UPGRADE};

pub fn scan(content: &str) -> Vec<PackageTrigger> {
    TriggerPosition::ALL
        .into_iter()
        .map(|position| (position, native_name(position)))
        .filter(|(_, name)| declares_function(content, name))
        .map(|(position, name)| PackageTrigger {
            position,
            name: name.to_owned(),
        })
        .collect()
}

fn native_name(position: TriggerPosition) -> &'static str {
    match position {
        TriggerPosition::PreInstall => PRE_INSTALL,
        TriggerPosition::PostInstall => POST_INSTALL,
        TriggerPosition::PreUpgrade => PRE_UPGRADE,
        TriggerPosition::PostUpgrade => POST_UPGRADE,
        TriggerPosition::PreRemove => PRE_REMOVE,
        TriggerPosition::PostRemove => POST_REMOVE,
    }
}

fn declares_function(content: &str, name: &str) -> bool {
    content.lines().any(|line| {
        if line.starts_with(char::is_whitespace) {
            return false;
        }

        let Some(rest) = line.strip_prefix(name) else {
            return false;
        };

        rest.trim_start().starts_with('(')
    })
}
