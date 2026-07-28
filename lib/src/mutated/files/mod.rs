use std::os::raw::c_void;

use upac_abi::DiffKind;
use upac_abi::error::ErrorKind;
use upac_abi::hook::{HookCancelToken, HookMessageFn};
use upac_abi::package::CPackageInfo;
use upac_abi::request::CFilesRequest;

pub use self::error::FilesError;

use crate::types::states::FilesStateId;

mod error;

pub struct FilesData<'a> {
    pub files: Vec<&'a str>,
    pub file_kind: DiffKind,
    pub file_package: &'a CPackageInfo,

    pub branch: &'a str,
    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

impl<'a> TryFrom<&'a CFilesRequest> for FilesData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CFilesRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let file_package = unsafe { request.file_package.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;
        let cancel_token = unsafe { request.base.hook_cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(FilesData {
            files: Vec::try_from(&request.files)?,
            file_kind: request.file_kind,
            file_package,

            branch: (&request.base.branch).try_into()?,
            tmp_path: (&request.tmp_path).try_into()?,

            hook_message: request.base.on_hook,
            hook_message_context: request.base.hook_ctx,

            hook_cancel_token: cancel_token,
        })
    }
}

pub fn run(data: FilesData) -> Result<(), (FilesStateId, FilesError)> {
    todo!()
}
