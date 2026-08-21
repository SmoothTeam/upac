// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

mod mutated;
mod search;
mod unmutated;

pub mod boot;
pub mod composefs;
pub mod config;
pub mod database;
pub mod deploy;
pub mod errors;
pub mod export;
pub mod fs;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod lock;
pub mod orchestrator;
pub mod plugin;
pub mod scripts;
