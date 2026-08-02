use std::os::raw::c_void;

use upac_abi::DiffKind;
use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CFilesRequest;

pub use self::error::FilesError;

use self::checkout::CheckoutStage;
use self::swap::SwapStage;
use self::transaction::TransactionStage;

use crate::orchestrator::{Context, Orchestrator, OrchestratorError};
use crate::types::states::FilesStateId;
use crate::types::{Branch, TmpPath};

mod checkout;
mod error;
mod swap;
mod transaction;

pub struct FilesData<'a> {
    pub files: Vec<&'a str>,
    pub file_kind: DiffKind,
    pub file_package: &'a CPackageInfo,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CFilesRequest> for FilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let file_package = unsafe { request.file_package.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;
        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(FilesData {
            files: Vec::try_from(&request.files)?,
            file_kind: request.file_kind,
            file_package,

            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            subject: (&request.subject).try_into()?,
            message: (&request.message).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<FilesError> {
    Orchestrator::new(vec![
        Box::new(TransactionStage),
        Box::new(CheckoutStage),
        Box::new(SwapStage),
    ])
}

pub fn run(data: FilesData) -> Result<(), (FilesStateId, FilesError)> {
    let mut context = Context::new();
    context.put(TmpPath(data.tmp_path.to_owned()));
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    let result = orchestrator
        .run_exclusive(&mut context, data.cancel_token)
        .map_err(|failure| match failure {
            OrchestratorError::Setup(lock_error) => (FilesStateId::Setup, FilesError::from(lock_error)),
            OrchestratorError::Stage(index, error) => (FilesStateId::from_stage_index(index), error),
        });

    data.cancel_token.reset();

    result
}
