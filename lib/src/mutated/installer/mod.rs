use std::os::raw::c_void;

use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn};
use upac_abi::request::CInstallRequest;

pub use self::error::InstallError;

use crate::types::PackageTemp;
use crate::types::states::InstallStateId;

mod error;

pub struct InstallData<'a> {
    pub packages: Vec<PackageTemp>,

    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub subject: &'a str,
    pub message: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CInstallRequest> for InstallData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CInstallRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(InstallData {
            packages: Vec::try_from(&request.packages)?,

            branch: (&request.base.branch).try_into()?,

            tmp_path: (&request.tmp_path).try_into()?,

            subject: (&request.subject).try_into()?,
            message: (&request.message).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            cancel_token: cancel_token,
        })
    }
}

pub fn run(data: InstallData) -> Result<(), (InstallStateId, InstallError)> {
    todo!()
}
