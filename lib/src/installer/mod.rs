use std::os::raw::c_void;

use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::package::CUnpackedPackage;

pub use self::error::InstallError;

use crate::types::states::InstallStateId;

mod error;

pub struct InstallData<'a> {
    pub packages: &'a [CUnpackedPackage],
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

pub fn run(data: InstallData) -> Result<(), (InstallStateId, InstallError)> {
    todo!()
}
