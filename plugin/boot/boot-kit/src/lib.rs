// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::consts::DLL_EXTENSION;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;

use libloading::Library;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::{CBootPluginEntryRequest, CBootPluginInstallRequest};
use upac_abi::response::CBootPluginInfo;

use upac_types::booter::BootResourceKind;
use upac_types::decoder::plugin_result;
use upac_types::request::{BootPluginEntryRequest, BootPluginInstallRequest};
use upac_types::response::BootPluginInfo;

use self::error::BooterError;

pub mod error;

macro_rules! load_symbol {
    ($library:expr, $name:literal) => {
        unsafe { load_symbol(&$library, $name)? }
    };
}

pub type BootPluginInfoFn = unsafe extern "C" fn() -> CBootPluginInfo;

pub type InstallFn = unsafe extern "C" fn(request: *const CBootPluginInstallRequest) -> i32;

pub type SetOneShotFn = unsafe extern "C" fn(request: *const CBootPluginEntryRequest) -> i32;

pub type SetDefaultFn = unsafe extern "C" fn(request: *const CBootPluginEntryRequest) -> i32;

pub trait BootBackend: Sized {
    type Error;

    fn new() -> Result<Self, Self::Error>;
    fn install(&mut self, request: &BootPluginInstallRequest) -> Result<(), Self::Error>;
    fn set_one_shot(&mut self, request: &BootPluginEntryRequest) -> Result<(), Self::Error>;
    fn set_default(&mut self, request: &BootPluginEntryRequest) -> Result<(), Self::Error>;
}

pub struct Booter {
    name: String,
    kind: BootResourceKind,
    install: InstallFn,
    set_one_shot: SetOneShotFn,
    set_default: SetDefaultFn,

    _library: Library,
}

impl Booter {
    pub fn load(library_path: &Path) -> Result<Self, BooterError> {
        let name = library_path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(BooterError::Load)?
            .to_owned();

        let library = unsafe { Library::new(library_path) }.map_err(|_| BooterError::Load)?;

        let plugin_info: BootPluginInfoFn = load_symbol!(library, "boot_plugin_info");
        let plugin_info = unsafe { plugin_info() };

        if plugin_info.abi_version != BOOT_ABI_VERSION {
            return Err(BooterError::AbiMismatch {
                got: plugin_info.abi_version,
                expected: BOOT_ABI_VERSION,
            });
        }

        let BootPluginInfo { kind, .. } =
            BootPluginInfo::try_from(&plugin_info).map_err(|_| BooterError::InvalidResponse)?;

        let install: InstallFn = load_symbol!(library, "install");
        let set_one_shot: SetOneShotFn = load_symbol!(library, "set_one_shot");
        let set_default: SetDefaultFn = load_symbol!(library, "set_default");

        Ok(Booter {
            name,
            kind,
            install,
            set_one_shot,
            set_default,
            _library: library,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> BootResourceKind {
        self.kind
    }

    pub fn install(&self, request: BootPluginInstallRequest) -> Result<(), BooterError> {
        let request: CBootPluginInstallRequest = request.into();

        let status = unsafe { (self.install)(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }

    pub fn set_one_shot(&self, request: BootPluginEntryRequest) -> Result<(), BooterError> {
        let request: CBootPluginEntryRequest = request.into();

        let status = unsafe { entry_fn(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }

    pub fn set_default(&self, request: BootPluginEntryRequest) -> Result<(), BooterError> {
        let request: CBootPluginEntryRequest = request.into();

        let status = unsafe { entry_fn(&request) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(BooterError::InvalidResponse, BooterError::Failed))
    }
}

pub struct Booters {
    booters: Vec<Booter>,
}

impl Booters {
    pub fn scan(plugin_dir: &Path) -> Result<Self, BooterError> {
        let mut booters = Vec::new();

        let entries = match read_dir(plugin_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == IoErrorKind::NotFound => return Ok(Booters { booters }),
            Err(error) => return Err(error.into()),
        };

        for entry in entries {
            let library_path = entry?.path();
            if library_path.extension() != Some(OsStr::new(DLL_EXTENSION)) {
                continue;
            }

            booters.push(Booter::load(&library_path)?);
        }

        Ok(Booters { booters })
    }

    pub fn get(&self, name: &str) -> Result<&Booter, BooterError> {
        let requested_name = name.replace('-', "_");

        let mut matching = self
            .booters
            .iter()
            .filter(|booter| booter.name.ends_with(&requested_name));

        let booter = matching
            .next()
            .ok_or_else(|| BooterError::UnknownName(name.to_owned()))?;

        if matching.next().is_some() {
            return Err(BooterError::AmbiguousName(name.to_owned()));
        }

        Ok(booter)
    }

    pub fn list(&self) -> impl Iterator<Item = (&str, BootResourceKind)> {
        self.booters.iter().map(|booter| (booter.name(), booter.kind()))
    }
}

unsafe fn load_symbol<Symbol: Copy>(library: &Library, name: &str) -> Result<Symbol, BooterError> {
    unsafe { library.get::<Symbol>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| BooterError::Symbol)
}
