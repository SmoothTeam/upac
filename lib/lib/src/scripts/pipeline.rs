// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Install,
    Uninstall,
    Update,
    Rollback,
    Commit,
    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timing {
    Pre,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineTrigger {
    pub operation: Operation,
    pub timing: Timing,
}

impl PipelineTrigger {
    pub fn pre(operation: Operation) -> Self {
        Self {
            operation,
            timing: Timing::Pre,
        }
    }

    pub fn post(operation: Operation) -> Self {
        Self {
            operation,
            timing: Timing::Post,
        }
    }
}
