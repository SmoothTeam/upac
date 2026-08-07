// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::collections::HashMap;

use crate::scripts::error::HookError;
use crate::scripts::file::HookFile;

pub struct TriggerEntry {
    pub name: String,
    pub hook_id: u16,
}

pub fn build_trigger_table(hooks: &[HookFile], format: &str) -> Result<Vec<TriggerEntry>, HookError> {
    let mut best: HashMap<&str, (u16, i32, bool)> = HashMap::new();

    for (index, hook) in hooks.iter().enumerate() {
        let Some(names) = hook.triggers.get(format) else {
            continue;
        };

        let hook_id = index as u16;

        for name in names {
            match best.get_mut(name.as_str()) {
                None => {
                    best.insert(name, (hook_id, hook.priority, false));
                }
                Some(winner) if hook.priority > winner.1 => {
                    *winner = (hook_id, hook.priority, false);
                }
                Some(winner) if hook.priority == winner.1 => {
                    winner.2 = true;
                }
                Some(_) => {}
            }
        }
    }

    let mut entries = Vec::with_capacity(best.len());

    for (name, (hook_id, _, tied)) in best {
        if tied {
            return Err(HookError::TriggerConflict(name.to_owned()));
        }

        entries.push(TriggerEntry {
            name: name.to_owned(),
            hook_id,
        });
    }

    Ok(entries)
}
