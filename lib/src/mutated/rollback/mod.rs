use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CRollbackRequest;

pub use self::error::RollbackError;

use crate::types::states::RollbackStateId;

mod error;

pub struct RollbackData<'a> {
    pub commit_hash: &'a str,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CRollbackRequest> for RollbackData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CRollbackRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(RollbackData {
            commit_hash: (&request.commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: RollbackData) -> Result<(), (RollbackStateId, RollbackError)> {
    todo!()
}
