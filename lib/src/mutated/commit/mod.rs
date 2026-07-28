use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CCommitRequest;

pub use self::error::CommitError;

use crate::types::states::CommitStateId;

mod error;

pub struct CommitData<'a> {
    pub message: &'a str,
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CCommitRequest> for CommitData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CCommitRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(CommitData {
            message: (&request.message).try_into()?,
            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: CommitData) -> Result<(), (CommitStateId, CommitError)> {
    todo!()
}
