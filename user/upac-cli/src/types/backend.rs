// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use strum::{AsRefStr, FromRepr};

#[derive(FromRepr, AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum BackendEvent {
    Verifying = 0,
    Extracting = 1,
    ReadingMeta = 2,
    Status = 3,
    Ready = 4,
    Failed = 5,
}

impl BackendEvent {
    pub fn format_message(&self, detail_string: &str) -> String {
        match self {
            Self::Verifying | Self::Extracting => {
                format!("{} {detail_string}...", gettextrs::gettext(self.as_ref()))
            }
            Self::Status => detail_string.to_string(),
            _ => gettextrs::gettext(self.as_ref()),
        }
    }
}

// ── Backend runtime config ────────────────────────────────────────────────────
// Loaded from a .toml file at runtime; describes one backend plugin.
// `so` is just the filename — the directory comes from Config::backends_dir.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub flags: Vec<String>,
    pub extensions: Vec<String>,
    pub so: String,
}
