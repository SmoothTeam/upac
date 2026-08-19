// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::MaybeUninit;

use libloading::Library;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::boot::{AbiVersionFn, CBootPluginRequest, ConfirmBootFn, ProbeFn, SetOneShotFn};
use upac_abi::error::ErrorKind;
use upac_abi::types::{CBorrowed, CSlice};

use crate::plugin::boot::error::BootPluginError;
use crate::plugin::boot::manifest::load_boot_plugin_manifests;

pub mod error;
pub mod manifest;

unsafe fn load_symbol<T: Copy>(library: &Library, name: &str) -> Result<T, BootPluginError> {
    unsafe { library.get::<T>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| BootPluginError::Symbol)
}

pub struct BootPlugin {
    probe: ProbeFn,
    set_one_shot: SetOneShotFn,
    confirm_boot: ConfirmBootFn,

    _library: Library,
}

impl BootPlugin {
    pub fn load(library_name: &str) -> Result<Self, BootPluginError> {
        let library = unsafe { Library::new(library_name) }.map_err(|_| BootPluginError::Load)?;

        let abi_version: AbiVersionFn = unsafe { load_symbol(&library, "abi_version")? };
        let probe: ProbeFn = unsafe { load_symbol(&library, "probe")? };
        let set_one_shot: SetOneShotFn = unsafe { load_symbol(&library, "set_one_shot")? };
        let confirm_boot: ConfirmBootFn = unsafe { load_symbol(&library, "confirm_boot")? };

        let got = unsafe { abi_version() };
        if got != BOOT_ABI_VERSION {
            return Err(BootPluginError::AbiMismatch {
                got,
                expected: BOOT_ABI_VERSION,
            });
        }

        Ok(BootPlugin {
            probe,
            set_one_shot,
            confirm_boot,
            _library: library,
        })
    }

    pub fn probes(&self) -> bool {
        unsafe { (self.probe)() == 1 }
    }

    pub fn set_one_shot(&self, entry_name: &str) -> Result<(), BootPluginError> {
        let request = CBootPluginRequest::new(CSlice::from_borrowed(entry_name.as_bytes()));
        let mut error = MaybeUninit::<ErrorKind>::uninit();

        let code = unsafe { (self.set_one_shot)(&request, error.as_mut_ptr()) };
        if code != 0 {
            return Err(BootPluginError::Reported(unsafe { error.assume_init() }));
        }

        Ok(())
    }

    pub fn confirm_boot(&self, entry_name: &str) -> Result<(), BootPluginError> {
        let request = CBootPluginRequest::new(CSlice::from_borrowed(entry_name.as_bytes()));
        let mut error = MaybeUninit::<ErrorKind>::uninit();

        let code = unsafe { (self.confirm_boot)(&request, error.as_mut_ptr()) };
        if code != 0 {
            return Err(BootPluginError::Reported(unsafe { error.assume_init() }));
        }

        Ok(())
    }
}

pub fn resolve_boot_plugin(
    boot_plugins_dir: &str, manifest_extension: &str, requested: Option<&str>,
) -> Result<BootPlugin, BootPluginError> {
    let manifests = load_boot_plugin_manifests(boot_plugins_dir, manifest_extension)?;

    match requested {
        Some(name) => {
            let manifest = manifests
                .get(name)
                .ok_or_else(|| BootPluginError::UnknownName(name.to_owned()))?;

            BootPlugin::load(&manifest.library)
        }
        None => {
            let mut claimants = Vec::new();
            for manifest in manifests.values() {
                let plugin = BootPlugin::load(&manifest.library)?;
                if plugin.probes() {
                    claimants.push(plugin);
                }
            }

            let mut claimants = claimants.into_iter();
            match (claimants.next(), claimants.next()) {
                (Some(plugin), None) => Ok(plugin),
                (None, _) => Err(BootPluginError::NoClaimant),
                (Some(_), Some(_)) => Err(BootPluginError::AmbiguousClaim),
            }
        }
    }
}
