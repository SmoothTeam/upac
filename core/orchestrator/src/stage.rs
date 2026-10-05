// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::TypeId;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use super::context::Context;

pub trait Stage<E> {
    fn requires(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn provides(&self) -> Vec<TypeId> {
        Vec::new()
    }

    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), E>;

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        Ok(())
    }
}
