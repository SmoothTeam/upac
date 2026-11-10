// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cell::RefCell;

use upac_types::CancelToken;
use upac_types::decoder::{PackageTriggers, TriggerPosition};
use upac_types::error::ErrorKind;

use upac_hooks::Hooks;
use upac_hooks::hook::Hook;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

pub struct HooksStage {
    position: TriggerPosition,
    executed: RefCell<Vec<Hook>>,
}

impl HooksStage {
    pub fn new(position: TriggerPosition) -> Self {
        Self {
            position,
            executed: RefCell::new(Vec::new()),
        }
    }
}

#[stage]
impl Stage<ErrorKind> for HooksStage {
    fn run(
        &self, context: &mut Context, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let hooks = Hooks::load()?.matching(self.position, context.get::<Vec<PackageTriggers>>()?)?;
        let total = hooks.len() as u64;

        for (position, mut hook) in hooks.into_iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(ErrorKind::Cancelled);
            }

            let result = hook.execute();
            self.executed.borrow_mut().push(hook);
            result?;

            progress(None, position as u64 + 1, total);
        }

        Ok(())
    }

    fn rollback(&self, _context: &mut Context) -> Result<(), ErrorKind> {
        for hook in self.executed.borrow_mut().iter_mut().rev() {
            hook.rollback()?;
        }

        Ok(())
    }
}
