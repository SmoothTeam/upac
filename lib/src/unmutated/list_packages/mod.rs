use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn};
use upac_abi::request::CListPackagesRequest;

pub use self::error::ListPackagesError;

use crate::types::PackageMeta;
use crate::types::states::ListPackagesStateId;

mod error;

pub struct ListPackagesData<'a> {
    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CListPackagesRequest> for ListPackagesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListPackagesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListPackagesData {
            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token: cancel_token,
        })
    }
}

pub fn run(data: ListPackagesData) -> Result<Vec<PackageMeta>, (ListPackagesStateId, ListPackagesError)> {
    todo!()
}
