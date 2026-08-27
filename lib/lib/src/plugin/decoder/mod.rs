// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::{Dependency, PackageMeta};

#[cfg(feature = "dynamic-plugins")]
use std::mem::MaybeUninit;
#[cfg(feature = "dynamic-plugins")]
use std::str::from_utf8;

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::ABI_VERSION;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::decoder::{AbiVersionFn, CDecodeRequest, CDecodeResponse, DecodeFn};

#[cfg(feature = "dynamic-plugins")]
use upac_abi::hook::CancelToken;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::types::{CBorrowed, CSlice};

#[cfg(feature = "dynamic-plugins")]
use crate::plugin::decoder::error::DecoderError;

pub mod error;
pub mod manifest;
pub mod triggers;
pub mod unpack;

/// A package decoded by a decoder plugin.
///
/// Plain owned data — available in every build configuration, including ones
/// without `dynamic-plugins`, so that callers and error types elsewhere in the
/// crate keep compiling.
pub struct DecodedPackage {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
    pub declarative_triggers: Vec<String>,
}

#[cfg(feature = "dynamic-plugins")]
unsafe fn load_symbol<T: Copy>(library: &Library, name: &str) -> Result<T, DecoderError> {
    unsafe { library.get::<T>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| DecoderError::Symbol)
}

/// A decoder plugin loaded from a shared object at runtime.
///
/// Exists only in builds with `dynamic-plugins`. There is currently no
/// compiled-in decoder path: decoders live outside this crate, so a build
/// without `dynamic-plugins` cannot decode packages at all. Once decoders are
/// rewritten in Rust, a `builtin-decoders` counterpart belongs here, mirroring
/// `static_plugins` on the boot side.
#[cfg(feature = "dynamic-plugins")]
pub struct Decoder {
    decode: DecodeFn,

    _library: Library,
}

#[cfg(feature = "dynamic-plugins")]
impl Decoder {
    pub fn load(library_name: &str) -> Result<Self, DecoderError> {
        let library = unsafe { Library::new(library_name) }.map_err(|_| DecoderError::Load)?;

        let abi_version: AbiVersionFn = unsafe { load_symbol(&library, "abi_version")? };
        let decode: DecodeFn = unsafe { load_symbol(&library, "decode")? };

        let got = unsafe { abi_version() };
        if got != ABI_VERSION {
            return Err(DecoderError::AbiMismatch {
                got,
                expected: ABI_VERSION,
            });
        }

        Ok(Decoder {
            decode,
            _library: library,
        })
    }

    pub fn decode(
        &self, package_path: &str, output_dir: &str, checksum: [u8; 32], cancel: &CancelToken,
    ) -> Result<DecodedPackage, DecoderError> {
        let request = CDecodeRequest::new(
            CSlice::from_borrowed(package_path.as_bytes()),
            CSlice::from_borrowed(output_dir.as_bytes()),
            checksum,
            cancel as *const CancelToken as *mut CancelToken,
        );

        let mut response = MaybeUninit::<CDecodeResponse>::uninit();

        let code = unsafe { (self.decode)(&request, response.as_mut_ptr()) };
        if code != 0 {
            return Err(DecoderError::Failed(code));
        }

        let response = unsafe { response.assume_init() };

        unsafe { response.validate() }?;

        let meta = PackageMeta::try_from(&response.meta)?;

        let dependencies = unsafe { response.dependencies.as_slice() }
            .iter()
            .map(Dependency::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let declarative_triggers = unsafe { response.declarative_triggers.as_slice() }
            .iter()
            .map(|trigger| unsafe { trigger.as_borrowed() })
            .map(|bytes| from_utf8(bytes).map(str::to_owned))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| DecoderError::InvalidResponse)?;

        Ok(DecodedPackage {
            meta,
            dependencies,
            declarative_triggers,
        })
    }
}
