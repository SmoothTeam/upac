use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CListPrefixRequest;

pub use self::error::ListPrefixError;

use crate::types::PrefixEntry;
use crate::types::states::ListPrefixStateId;

mod error;

pub struct ListPrefixData<'a> {
    pub branch: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CListPrefixRequest> for ListPrefixData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CListPrefixRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(ListPrefixData {
            branch: (&request.base.branch).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: ListPrefixData) -> Result<Vec<PrefixEntry>, (ListPrefixStateId, ListPrefixError)> {
    todo!()
}
