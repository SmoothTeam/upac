use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CDiffFilesRequest;

pub struct DiffFilesData<'a> {
    pub from_commit_hash: Option<&'a str>,
    pub to_commit_hash: Option<&'a str>,

    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CDiffFilesRequest> for DiffFilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CDiffFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        let from_commit_hash = if request.from_commit_hash.ptr.is_null() {
            None
        } else {
            Some((&request.from_commit_hash).try_into()?)
        };
        let to_commit_hash = if request.to_commit_hash.ptr.is_null() {
            None
        } else {
            Some((&request.to_commit_hash).try_into()?)
        };

        Ok(DiffFilesData {
            from_commit_hash,
            to_commit_hash,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}
