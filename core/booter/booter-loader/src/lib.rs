// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::plugin::{BootResourceKindFn, ConfirmBootFn, InstallFn, SetOneShotFn};
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

use upac_types::booter::{BootResourceKind, BuiltinBooter};
use upac_types::plugin::plugin_result;
use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};

use self::error::BootPluginError;

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "dynamic-plugins")]
use self::manifest::BootPluginManifests;

#[cfg(feature = "dynamic-plugins")]
mod dynamic_link;
#[cfg(feature = "dynamic-plugins")]
mod manifest;

pub mod error;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

#[cfg(feature = "builtin-booters")]
const BUILTIN_BOOTERS: &[BuiltinBooter] = &[
    #[cfg(feature = "builtin-uki")]
    upac_boot_uki::BOOTER,
    #[cfg(feature = "builtin-systemd-boot")]
    upac_boot_systemd_boot::BOOTER,
    #[cfg(feature = "builtin-grub")]
    upac_boot_grub::BOOTER,
    #[cfg(feature = "builtin-refind")]
    upac_boot_refind::BOOTER,
];

pub struct BootPlugins {
    #[cfg(feature = "dynamic-plugins")]
    manifests: BootPluginManifests,
}

impl BootPlugins {
    pub fn new() -> Result<Self, BootPluginError> {
        Ok(BootPlugins {
            #[cfg(feature = "dynamic-plugins")]
            manifests: BootPluginManifests::new()?,
        })
    }

    #[allow(unreachable_code)]
    pub fn load(&self, name: &str) -> Result<BootPlugin, BootPluginError> {
        #[cfg(feature = "builtin-booters")]
        if let Some(builtin) = BUILTIN_BOOTERS.iter().find(|builtin| builtin.name == name) {
            return Ok(BootPlugin::from(builtin));
        }

        #[cfg(feature = "dynamic-plugins")]
        return dynamic_link::load_boot_plugin_dynamic(&self.manifests, name);

        #[cfg(feature = "builtin-booters")]
        return Err(BootPluginError::UnknownName(name.to_owned()));

        #[cfg(not(any(feature = "dynamic-plugins", feature = "builtin-booters")))]
        {
            let _ = name;
            Err(BootPluginError::NoClaimant)
        }
    }
}

pub struct BootPlugin {
    set_one_shot: SetOneShotFn,
    confirm_boot: ConfirmBootFn,
    install: InstallFn,
    boot_resource_kind: BootResourceKindFn,

    #[cfg(feature = "dynamic-plugins")]
    _library: Option<Library>,
}

impl From<&BuiltinBooter> for BootPlugin {
    fn from(builtin: &BuiltinBooter) -> Self {
        BootPlugin {
            set_one_shot: builtin.set_one_shot,
            confirm_boot: builtin.confirm_boot,
            install: builtin.install,
            boot_resource_kind: builtin.boot_resource_kind,

            #[cfg(feature = "dynamic-plugins")]
            _library: None,
        }
    }
}

impl BootPlugin {
    pub fn boot_resource_kind(&self) -> Result<BootResourceKind, BootPluginError> {
        let kind = unsafe { (self.boot_resource_kind)() };

        BootResourceKind::try_from(kind).map_err(|_| BootPluginError::InvalidResponse)
    }

    pub fn set_one_shot(&self, request: BootPluginSetOneShotRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginSetOneShotRequest = request.into();

        let status = unsafe { (self.set_one_shot)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BootPluginError::InvalidResponse, BootPluginError::Failed))
    }

    pub fn confirm_boot(&self, request: BootPluginConfirmSuccessBootRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginConfirmSuccessBootRequest = request.into();

        let status = unsafe { (self.confirm_boot)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BootPluginError::InvalidResponse, BootPluginError::Failed))
    }

    pub fn install(&self, request: BootPluginInstallRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginInstallRequest = request.into();

        let status = unsafe { (self.install)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BootPluginError::InvalidResponse, BootPluginError::Failed))
    }
}
