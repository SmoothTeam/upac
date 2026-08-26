// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::os::raw::c_void;

use upac_abi::FsKind;
use upac_abi::error::ErrorKind;
use upac_abi::hook::{CancelToken, HookMessageFn};
use upac_abi::setup::CSetupExistingRequest;

use upac_types::PartitionMount;

pub struct SetupData<'a> {
    pub esp_device: &'a str,
    pub deploy_device: &'a str,
    pub deploy_fs: FsKind,
    pub extra_mounts: Vec<PartitionMount>,

    pub mount_point: Option<&'a str>,
    pub source_dir: &'a str,
    pub meta_filename: Option<&'a str>,
    pub empty_config: bool,
    pub pinned: bool,
    pub boot_plugin: Option<&'a str>,

    pub hook_message: Option<HookMessageFn>,
    pub hook_message_context: *mut c_void,

    pub cancel_token: &'a CancelToken,
}

impl<'a> TryFrom<&'a CSetupExistingRequest> for SetupData<'a> {
    type Error = ErrorKind;

    fn try_from(request: &'a CSetupExistingRequest) -> Result<Self, ErrorKind> {
        unsafe { request.validate()? };

        let cancel_token = unsafe { request.base.base.cancel_token.as_ref() }.ok_or(ErrorKind::InvalidEntry)?;

        Ok(SetupData {
            esp_device: (&request.esp_device).try_into()?,
            deploy_device: (&request.deploy_device).try_into()?,
            deploy_fs: request.deploy_fs,
            extra_mounts: Vec::try_from(&request.extra_mounts)?,

            mount_point: (&request.base.mount_point).try_into()?,
            source_dir: (&request.base.source_dir).try_into()?,
            meta_filename: (&request.base.meta_filename).try_into()?,
            empty_config: request.base.empty_config,
            pinned: request.base.pinned,
            boot_plugin: (&request.base.boot_plugin).try_into()?,

            hook_message: request.base.base.on_hook,
            hook_message_context: request.base.base.hook_ctx,

            cancel_token,
        })
    }
}
