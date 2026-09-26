// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::ErrorKind;
use upac_abi::plugin::{BootResourceKind, BootResourceKindFn, ConfirmBootFn, InstallFn, SetOneShotFn};
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

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
#[cfg(feature = "builtin-booters")]
mod static_link;

pub mod entry;
pub mod error;
pub mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

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
        if let Ok(plugin) = static_link::load_boot_plugin_static(name) {
            return Ok(plugin);
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

impl BootPlugin {
    pub fn boot_resource_kind(&self) -> Result<BootResourceKind, BootPluginError> {
        let kind = unsafe { (self.boot_resource_kind)() };

        BootResourceKind::try_from(kind).map_err(BootPluginError::Reported)
    }

    pub fn set_one_shot(&self, request: BootPluginSetOneShotRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginSetOneShotRequest = request.into();

        let response_code = unsafe { (self.set_one_shot)(&request) };
        if response_code != 0 {
            return Err(BootPluginError::Reported(
                ErrorKind::try_from(response_code as u32).unwrap_or(ErrorKind::InvalidEntry),
            ));
        }

        Ok(())
    }

    pub fn confirm_boot(&self, request: BootPluginConfirmSuccessBootRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginConfirmSuccessBootRequest = request.into();

        let response_code = unsafe { (self.confirm_boot)(&request) };
        if response_code != 0 {
            return Err(BootPluginError::Reported(
                ErrorKind::try_from(response_code as u32).unwrap_or(ErrorKind::InvalidEntry),
            ));
        }

        Ok(())
    }

    pub fn install(&self, request: BootPluginInstallRequest) -> Result<(), BootPluginError> {
        let request: CBootPluginInstallRequest = request.into();

        let response_code = unsafe { (self.install)(&request) };
        if response_code != 0 {
            return Err(BootPluginError::Reported(
                ErrorKind::try_from(response_code as u32).unwrap_or(ErrorKind::InvalidEntry),
            ));
        }

        Ok(())
    }
}
