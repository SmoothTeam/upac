use std::os::raw::c_void;

use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::package::CPackageInfo;
use upac_abi::DiffKind;

pub use self::error::FilesError;

use crate::types::states::FilesStateId;

mod error;

pub struct FilesData<'a> {
    pub files: &'a [&'a str],
    pub file_kind: DiffKind,
    pub file_package: &'a CPackageInfo,

    pub branch: &'a str,
    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

pub fn run(data: FilesData) -> Result<(), (FilesStateId, FilesError)> {
    todo!()
}
