// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use super::error::HookError;
use super::hook::Hook;

#[cfg(test)]
#[path = "../tests/inline/triggers.rs"]
mod tests;

pub(crate) fn build_trigger_table<'hooks>(
    hooks: &'hooks [Hook], format: &str,
) -> Result<HashMap<&'hooks str, usize>, HookError> {
    let mut best: HashMap<&str, (usize, i32, bool)> = HashMap::new();

    for (hook_index, hook) in hooks.iter().enumerate() {
        let Some(names) = hook.triggers.get(format) else {
            continue;
        };

        for name in names {
            match best.get_mut(name.as_str()) {
                None => {
                    best.insert(name, (hook_index, hook.priority, false));
                }
                Some(winner) if hook.priority > winner.1 => {
                    *winner = (hook_index, hook.priority, false);
                }
                Some(winner) if hook.priority == winner.1 => {
                    winner.2 = true;
                }
                Some(_) => {}
            }
        }
    }

    let mut table = HashMap::with_capacity(best.len());

    for (name, (hook_index, _, tied)) in best {
        if tied {
            return Err(HookError::TriggerConflict(name.to_owned()));
        }

        table.insert(name, hook_index);
    }

    Ok(table)
}
