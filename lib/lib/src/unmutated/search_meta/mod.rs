use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CSearchMetaRequest;

pub use self::error::SearchMetaError;

use self::searching::SearchingStage;

use crate::orchestrator::{Context, Orchestrator};
use crate::types::errors::CommonError;
use crate::types::states::SearchMetaStateId;
use crate::types::{Branch, PackageMeta};

mod error;
mod searching;

pub struct SearchMetaData<'a> {
    pub search: &'a str,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CSearchMetaRequest> for SearchMetaData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CSearchMetaRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(SearchMetaData {
            search: (&request.search).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<SearchMetaError> {
    Orchestrator::new(vec![Box::new(SearchingStage)])
}

pub fn run(data: SearchMetaData) -> Result<Vec<PackageMeta>, (SearchMetaStateId, SearchMetaError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (SearchMetaStateId::from_stage_index(index), error))?;

    context.take::<Vec<PackageMeta>>().ok_or((
        SearchMetaStateId::Setup,
        SearchMetaError::from(CommonError::MissingResult),
    ))
}
