// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::CancelToken;
use upac_types::error::ErrorKind;
use upac_types::transaction::TransactionKind;

use upac_deploy::working::{CommittedPrefix, WorkingPrefix};

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use crate::layout::genesis::SUBJECT;

pub struct CommitStage;

#[stage]
impl Stage<ErrorKind> for CommitStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let working = context.take::<WorkingPrefix>()?;

        let committed = working.commit(TransactionKind::Bootstrap, SUBJECT.to_owned(), None)?;
        context.put::<CommittedPrefix>(committed);

        Ok(())
    }
}
