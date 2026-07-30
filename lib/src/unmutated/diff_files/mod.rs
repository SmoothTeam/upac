use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CDiffFilesRequest;

pub use self::error::DiffFilesError;

use crate::types::DiffFileEntry;
use crate::types::states::DiffFilesStateId;

mod error;

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

        Ok(DiffFilesData {
            from_commit_hash: (&request.from_commit_hash).try_into()?,
            to_commit_hash: (&request.to_commit_hash).try_into()?,

            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: DiffFilesData) -> Result<Vec<DiffFileEntry>, (DiffFilesStateId, DiffFilesError)> {
    todo!()
}
