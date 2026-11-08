// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::BOOT_ABI_VERSION;
#[cfg(feature = "dynamic-plugins")]
use upac_abi::plugin::BootPluginAbiVersionFn;
use upac_abi::plugin::{BootResourceKindFn, ConfirmBootFn, InstallFn, SetOneShotFn};
#[cfg(feature = "host")]
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

#[cfg(any(feature = "host", feature = "plugin"))]
use upac_types::booter::BootResourceKind;
#[cfg(feature = "host")]
use upac_types::plugin::plugin_result;
#[cfg(feature = "host")]
use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};

#[cfg(feature = "host")]
use self::error::BooterError;
#[cfg(feature = "dynamic-plugins")]
use self::manifest::BooterManifest;

#[cfg(feature = "host")]
pub mod error;
#[cfg(feature = "host")]
pub mod manifest;

#[cfg(feature = "dynamic-plugins")]
macro_rules! load_symbol {
    ($library:expr, $name:literal) => {
        unsafe { load_symbol(&$library, $name)? }
    };
}

#[cfg(feature = "plugin")]
pub trait BootBackend: Sized {
    type Error;

    fn new() -> Result<Self, Self::Error>;
    fn boot_resource_kind() -> BootResourceKind;
    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), Self::Error>;
    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), Self::Error>;
    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinBooter {
    pub name: &'static str,
    pub boot_resource_kind: BootResourceKindFn,
    pub set_one_shot: SetOneShotFn,
    pub confirm_boot: ConfirmBootFn,
    pub install: InstallFn,
}

#[cfg(feature = "host")]
pub struct Booter {
    name: String,
    boot_resource_kind: BootResourceKindFn,
    set_one_shot: SetOneShotFn,
    confirm_boot: ConfirmBootFn,
    install: InstallFn,

    #[cfg(feature = "dynamic-plugins")]
    _library: Option<Library>,
}

#[cfg(feature = "host")]
impl Booter {
    #[cfg(feature = "dynamic-plugins")]
    pub fn load(manifest: &BooterManifest) -> Result<Self, BooterError> {
        let library = unsafe { Library::new(&manifest.library) }.map_err(|_| BooterError::Load)?;

        let abi_version: BootPluginAbiVersionFn = load_symbol!(library, "boot_abi_version");
        let boot_resource_kind: BootResourceKindFn = load_symbol!(library, "boot_resource_kind");
        let set_one_shot: SetOneShotFn = load_symbol!(library, "set_one_shot");
        let confirm_boot: ConfirmBootFn = load_symbol!(library, "confirm_boot");
        let install: InstallFn = load_symbol!(library, "install");

        let got_abi_version = unsafe { abi_version() };
        if got_abi_version != BOOT_ABI_VERSION {
            return Err(BooterError::AbiMismatch {
                got: got_abi_version,
                expected: BOOT_ABI_VERSION,
            });
        }

        Ok(Booter {
            name: manifest.name.clone(),
            boot_resource_kind,
            set_one_shot,
            confirm_boot,
            install,
            _library: Some(library),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn boot_resource_kind(&self) -> Result<BootResourceKind, BooterError> {
        let kind = unsafe { (self.boot_resource_kind)() };

        BootResourceKind::try_from(kind).map_err(|_| BooterError::InvalidResponse)
    }

    pub fn set_one_shot(&self, request: BootPluginSetOneShotRequest) -> Result<(), BooterError> {
        let request: CBootPluginSetOneShotRequest = request.into();

        let status = unsafe { (self.set_one_shot)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }

    pub fn confirm_boot(&self, request: BootPluginConfirmSuccessBootRequest) -> Result<(), BooterError> {
        let request: CBootPluginConfirmSuccessBootRequest = request.into();

        let status = unsafe { (self.confirm_boot)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }

    pub fn install(&self, request: BootPluginInstallRequest) -> Result<(), BooterError> {
        let request: CBootPluginInstallRequest = request.into();

        let status = unsafe { (self.install)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }
}

#[cfg(feature = "host")]
impl From<&BuiltinBooter> for Booter {
    fn from(builtin: &BuiltinBooter) -> Self {
        Booter {
            name: builtin.name.to_owned(),
            boot_resource_kind: builtin.boot_resource_kind,
            set_one_shot: builtin.set_one_shot,
            confirm_boot: builtin.confirm_boot,
            install: builtin.install,

            #[cfg(feature = "dynamic-plugins")]
            _library: None,
        }
    }
}

#[cfg(feature = "dynamic-plugins")]
unsafe fn load_symbol<Symbol: Copy>(library: &Library, name: &str) -> Result<Symbol, BooterError> {
    unsafe { library.get::<Symbol>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| BooterError::Symbol)
}
