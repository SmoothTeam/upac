use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn, Message, MessageHook};
use upac_abi::request::CListHistoryRequest;

pub use self::error::ListHistoryError;

use self::fetching::FetchingStage;

use crate::orchestrator::{Context, Orchestrator};
use crate::types::errors::CommonError;
use crate::types::states::ListHistoryStateId;
use crate::types::{Branch, HistoryEntry};

mod error;
mod fetching;

pub struct ListHistoryData<'a> {
    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CListHistoryRequest> for ListHistoryData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListHistoryRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListHistoryData {
            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token,
        })
    }
}

fn assemble() -> Orchestrator<ListHistoryError> {
    Orchestrator::new(vec![Box::new(FetchingStage)])
}

pub fn run(data: ListHistoryData) -> Result<Vec<HistoryEntry>, (ListHistoryStateId, ListHistoryError)> {
    let mut context = Context::new();
    context.put(Branch(data.branch.to_owned()));
    context.put(Box::new(Message::new(data.hook_message, data.hook_message_context)) as Box<dyn MessageHook>);

    let mut orchestrator = assemble();

    orchestrator
        .run_concurrent(&mut context, data.cancel_token)
        .map_err(|(index, error)| (ListHistoryStateId::from_stage_index(index), error))?;

    context.take::<Vec<HistoryEntry>>().ok_or((
        ListHistoryStateId::Setup,
        ListHistoryError::from(CommonError::MissingResult),
    ))
}
