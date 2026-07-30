use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CListCommitRequest;

pub use self::error::ListCommitError;

use crate::types::CommitEntry;
use crate::types::states::ListCommitStateId;

mod error;

pub struct ListCommitData<'a> {
    pub prefix_digest: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CListCommitRequest> for ListCommitData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListCommitRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListCommitData {
            prefix_digest: (&request.prefix_digest).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: ListCommitData) -> Result<Vec<CommitEntry>, (ListCommitStateId, ListCommitError)> {
    todo!()
}
