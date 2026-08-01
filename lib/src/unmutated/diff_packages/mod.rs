use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn};
use upac_abi::request::CDiffPackagesRequest;

pub use self::error::DiffPackagesError;

use crate::types::DiffPackageEntry;
use crate::types::states::DiffPackagesStateId;

mod error;

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

            cancel_token: cancel_token,
        })
    }
}

pub fn run(data: DiffPackagesData) -> Result<Vec<DiffPackageEntry>, (DiffPackagesStateId, DiffPackagesError)> {
    todo!()
}
