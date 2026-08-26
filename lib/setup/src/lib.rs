// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

pub mod data;
pub mod error;
pub mod format;
pub mod genesis;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
pub mod meta;
pub mod partition;
pub mod target;
