// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub mod config;
pub mod diff;
pub mod error;
pub mod file;
pub mod overlay;
pub mod repository;
