// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclarativeTrigger {
    pub format: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoderTrigger {
    PreInstall,
    PostInstall,
    PreUpgrade,
    PostUpgrade,
    PreRemove,
    PostRemove,
}

impl DecoderTrigger {
    pub const ALL: [DecoderTrigger; 6] = [
        DecoderTrigger::PreInstall,
        DecoderTrigger::PostInstall,
        DecoderTrigger::PreUpgrade,
        DecoderTrigger::PostUpgrade,
        DecoderTrigger::PreRemove,
        DecoderTrigger::PostRemove,
    ];
}
