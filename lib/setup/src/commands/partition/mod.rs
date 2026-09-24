// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::path::PathBuf;

use upac_macro::ContextValue;

pub mod add;
pub mod error;
pub mod gpt;
pub mod table;

#[derive(ContextValue)]
pub(crate) struct RequestedDevice(pub PathBuf);
