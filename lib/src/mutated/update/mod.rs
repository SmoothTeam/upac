use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::request::CUpdateRequest;

pub use self::error::UpdateError;

use crate::types::PackageTemp;
use crate::types::states::UpdateStateId;

mod error;

pub struct UpdateData<'a> {
    pub packages: Vec<PackageTemp>,
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CUpdateRequest> for UpdateData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CUpdateRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(UpdateData {
            packages: Vec::try_from(&request.packages)?,
            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: UpdateData) -> Result<(), (UpdateStateId, UpdateError)> {
    todo!()
}
