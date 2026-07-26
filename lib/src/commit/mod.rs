use std::os::raw::c_void;

use upac_abi::hook::{HookCancelToken, HookMessageFn};

pub use self::error::CommitError;

use crate::types::states::CommitStateId;

mod error;

pub struct CommitData<'a> {
    pub message: &'a str,
    pub branch: &'a str,

    pub tmp_path: &'a str,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub hook_cancel_token: &'a HookCancelToken,
}

pub fn run(data: CommitData) -> Result<(), (CommitStateId, CommitError)> {
    todo!()
}
