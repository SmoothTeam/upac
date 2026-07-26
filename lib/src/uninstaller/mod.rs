use std::os::raw::c_void;

use upac_abi::hook::{Cancel, CancelHook, HookCancelToken, HookMessageFn, Message, MessageHook};
use crate::deploy::{Deploy, DeployMode};
use crate::orchestrator::{Context, Orchestrator};
use crate::types::lock::Lock;
use crate::types::states::UninstallStateId;
use crate::types::{Branch, PackageEntry, Targets, TmpPath};

pub use self::error::UninstallError;

use self::boot_option::BootOptionStage;
use self::build::BuildStage;
use self::commit::CommitStage;
use self::config_merge::ConfigMergeStage;
use self::prepare_boot::PrepareBootStage;
use self::preparation::PreparationStage;

mod boot_option;
mod build;
mod commit;
mod config_merge;
mod error;
mod prepare_boot;
mod preparation;

pub struct UninstallPackage<'a> {
    pub name: &'a str,
    pub arch: &'a str,
    pub arch_sub: Option<&'a str>,
}

pub struct UninstallData<'a> {
    pub packages: &'a [UninstallPackage<'a>],
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

fn assemble() -> Orchestrator<UninstallError> {
    Orchestrator::new(vec![
        Box::new(PreparationStage),
        Box::new(BuildStage),
        Box::new(CommitStage),
        Box::new(ConfigMergeStage),
        Box::new(PrepareBootStage),
        Box::new(BootOptionStage),
    ])
}

pub fn run(data: UninstallData) -> Result<(), (UninstallStateId, UninstallError)> {
    let _lock = Lock::acquire().map_err(|error| (UninstallStateId::Setup, UninstallError::from(error)))?;
    let deploy = Deploy::new(DeployMode::ReadWrite).map_err(|error| (UninstallStateId::Setup, UninstallError::from(error)))?;

    let targets = Targets(
        data.packages
            .iter()
            .map(|package| PackageEntry {
                name: package.name.to_owned(),
                arch: package.arch.to_owned(),
                arch_sub: package.arch_sub.map(str::to_owned),
            })
            .collect(),
    );

    let mut context = Context::new();
    context.put(targets);
    context.put(deploy);
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);
    context.put(Box::new(Cancel::new(data.hook_cancel_token as *const HookCancelToken)) as Box<dyn CancelHook>);

    let orchestrator = assemble();

    let result = if orchestrator.validate(&context).is_err() {
        Err((UninstallStateId::Setup, UninstallError::UninstallFailed))
    } else {
        orchestrator
            .run(&mut context)
            .map_err(|(index, error)| (UninstallStateId::from_stage_index(index), error))
    };

    if let Some(cancel) = context.get::<Box<dyn CancelHook>>() {
        cancel.reset();
    }

    result
}
