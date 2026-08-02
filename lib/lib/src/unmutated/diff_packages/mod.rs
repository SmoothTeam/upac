use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CDiffPackagesRequest;

pub use self::error::DiffPackagesError;

use self::comparing::ComparingStage;
use self::preparing::PreparingStage;

use crate::orchestrator::{Context, Orchestrator};
use crate::types::errors::CommonError;
use crate::types::states::DiffPackagesStateId;
use crate::types::{Branch, DiffPackageEntry};

mod comparing;
mod error;
mod preparing;

pub struct DiffPackagesData<'a> {
    pub from_commit_hash: Option<&'a str>,
    pub to_commit_hash: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CDiffPackagesRequest> for DiffPackagesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffPackagesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(DiffPackagesData {
            from_commit_hash: (&request.from_commit_hash).try_into()?,
            to_commit_hash: (&request.to_commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<DiffPackagesError> {
    Orchestrator::new(vec![Box::new(PreparingStage), Box::new(ComparingStage)])
}

pub fn run(data: DiffPackagesData) -> Result<Vec<DiffPackageEntry>, (DiffPackagesStateId, DiffPackagesError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (DiffPackagesStateId::from_stage_index(index), error))?;

    context.take::<Vec<DiffPackageEntry>>().ok_or((
        DiffPackagesStateId::Setup,
        DiffPackagesError::from(CommonError::MissingResult),
    ))
}
