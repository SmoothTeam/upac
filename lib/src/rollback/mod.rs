use std::os::raw::c_void;

use upac_abi::hook::{HookCancelToken, HookMessageFn};

pub use self::error::RollbackError;

use crate::types::states::RollbackStateId;

mod error;

pub struct RollbackData<'a> {
    pub commit_hash: &'a str,
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

pub fn run(data: RollbackData) -> Result<(), (RollbackStateId, RollbackError)> {
    todo!()
}
