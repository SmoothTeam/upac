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

use super::CommitInfo;

pub struct CommitStage {
    pub kind: TransactionKind,
}

#[stage]
impl Stage<ErrorKind> for CommitStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let working = context.take::<WorkingPrefix>()?;
        let commit_info = context.get::<CommitInfo>()?;

        let committed = working.commit(self.kind, commit_info.subject.clone(), commit_info.message.clone())?;
        context.put::<CommittedPrefix>(committed);

        Ok(())
    }
}
