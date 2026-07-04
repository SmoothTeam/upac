use std::os::raw::c_void;

use crate::ffi::{CancelToken, HookFn};
use crate::types::errors::{ErrorCode, UninstallError, to_code};
use crate::types::machine::{Context, Orchestrator};
use crate::types::{Branch, Lock, PackageEntry, RepoPath, RootPath, Targets, TmpPath};

use self::checkout::CheckoutStage;
use self::preparation::PreparationStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

mod checkout;
mod merge;
mod preparation;
mod swap;
mod transaction;

pub struct UninstallPackage<'a> {
    pub name: &'a str,
    pub arch: &'a str,
    pub arch_sub: Option<&'a str>,
}

pub struct UninstallData<'a> {
    pub packages: &'a [UninstallPackage<'a>],
    pub branch: &'a str,
    pub repo_path: &'a str,
    pub root_path: &'a str,
    pub tmp_path: &'a str,
    pub on_hook: Option<HookFn>,
    pub hook_ctx: *mut c_void,
    pub cancel_token: &'a CancelToken,
}

fn assemble() -> Orchestrator<UninstallError> {
    Orchestrator::new(vec![
        Box::new(PreparationStage),
        Box::new(TransactionStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
    ])
}

pub fn run(data: UninstallData) -> i32 {
    let _lock = match Lock::acquire() {
        Ok(lock) => lock,
        Err(error) => return ErrorCode::from(error) as i32,
    };

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
    context.put(RepoPath(data.repo_path.to_owned()));
    context.put(RootPath(data.root_path.to_owned()));
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));

    let orchestrator = assemble();

    if orchestrator.validate(&context).is_err() {
        return ErrorCode::Unexpected as i32;
    }

    to_code(orchestrator.run(&mut context))
}
