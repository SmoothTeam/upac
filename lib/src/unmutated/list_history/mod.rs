use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn};
use upac_abi::request::CListHistoryRequest;

pub use self::error::ListHistoryError;

use crate::types::HistoryEntry;
use crate::types::states::ListHistoryStateId;

mod error;

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

            cancel_token: cancel_token,
        })
    }
}

pub fn run(data: ListHistoryData) -> Result<Vec<HistoryEntry>, (ListHistoryStateId, ListHistoryError)> {
    todo!()
}
