// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::TypeId;

use upac_macro::stage;

use upac_types::CancelToken;

use upac_orchestrator::context::Context;
use upac_orchestrator::error::PipelineError;
use upac_orchestrator::stage::Stage;

struct Config;

struct Packages;

struct Report;

struct ContextStage;

#[stage]
impl Stage<PipelineError> for ContextStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), PipelineError> {
        context.get::<Config>()?;
        context.get::<Config>()?;

        let packages = context.take::<Packages>()?;
        context.put(packages);
        context.put(Report);
        context.push::<u32>(7);
        context.push::<u32>(8);

        Ok(())
    }
}

#[test]
fn reads_are_required_once_each() {
    assert_eq!(
        ContextStage.requires(),
        vec![TypeId::of::<Config>(), TypeId::of::<Packages>()]
    );
}

#[test]
fn puts_and_pushes_are_provided_once_each() {
    assert_eq!(
        ContextStage.provides(),
        vec![
            TypeId::of::<Packages>(),
            TypeId::of::<Report>(),
            TypeId::of::<Vec<u32>>()
        ]
    );
}

#[test]
fn a_taken_slot_put_back_is_both_required_and_provided() {
    assert!(ContextStage.requires().contains(&TypeId::of::<Packages>()));
    assert!(ContextStage.provides().contains(&TypeId::of::<Packages>()));
}
