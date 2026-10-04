// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

pub mod mutated;
pub mod setup;
pub mod unmutated;

macro_rules! impl_command_state {
    ($name:ident, $domain:ident) => {
        impl crate::traits::CommandState for $name {
            const DOMAIN: $crate::error::ErrorDomain = $crate::error::ErrorDomain::$domain;
            const VALIDATION: Self = $name::Setup;

            fn as_u32(self) -> u32 {
                self as u32
            }

            fn from_stage_index(index: usize) -> Self {
                $name::from_stage_index(index)
            }
        }
    };
}
pub(crate) use impl_command_state;
