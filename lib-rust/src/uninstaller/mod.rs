use std::os::raw::c_void;

use gio::Cancellable;
use glib::prelude::ObjectType;

use crate::ffi::{HookCancelToken, HookFn};
use crate::types::errors::{ErrorCode, UninstallError, to_code};
use crate::types::machine::{Context, Orchestrator};
use crate::types::{
    Branch, HookCancelHandle, HookMessageHandle, Lock, PackageEntry, RepoPath, RootPath, Targets, TmpPath,
};

use self::checkout::CheckoutStage;
use self::merge::MergeStage;
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

    pub hook_message: Option<HookFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

fn assemble() -> Orchestrator<UninstallError> {
    Orchestrator::new(vec![
        Box::new(PreparationStage),
        Box::new(TransactionStage),
        Box::new(MergeStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
    ])
}

unsafe extern "C" fn cancel_via_gcancellable(ctx: *mut c_void) {
    unsafe { gio::ffi::g_cancellable_cancel(ctx as *mut gio::ffi::GCancellable) };
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

    let cancellable = Cancellable::new();
    data.hook_cancel_token
        .bind(cancel_via_gcancellable, cancellable.as_ptr() as *mut c_void);

    let mut context = Context::new();
    context.put(targets);
    context.put(RepoPath(data.repo_path.to_owned()));
    context.put(RootPath(data.root_path.to_owned()));
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(HookMessageHandle::new(data.hook_message, data.hook_message_context));
    context.put(HookCancelHandle::new(data.hook_cancel_token as *const HookCancelToken));
    context.put(cancellable);

    let orchestrator = assemble();

    let code = if orchestrator.validate(&context).is_err() {
        ErrorCode::Unexpected as i32
    } else {
        to_code(orchestrator.run(&mut context))
    };

    data.hook_cancel_token.reset();

    code
}
