// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

#[cfg(feature = "host")]
use std::mem::MaybeUninit;
#[cfg(feature = "host")]
use std::os::raw::c_void;
#[cfg(feature = "host")]
use std::path::Path;
#[cfg(feature = "host")]
use std::slice::from_raw_parts;

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "host")]
use zerocopy::IntoBytes;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::DECODER_ABI_VERSION;
use upac_abi::plugin::DecodeFn;
#[cfg(feature = "dynamic-plugins")]
use upac_abi::plugin::DecodePluginAbiVersionFn;
#[cfg(feature = "host")]
use upac_abi::request::decoder::CDecodeRequest;
#[cfg(feature = "host")]
use upac_abi::response::decoder::CDecodeResponse;

#[cfg(feature = "host")]
use upac_composefs::Repo;

#[cfg(feature = "host")]
use upac_types::CancelToken;
#[cfg(feature = "host")]
use upac_types::decoder::DecodeError;
#[cfg(feature = "host")]
use upac_types::package::PackageChecksum;
#[cfg(feature = "host")]
use upac_types::plugin::{plugin_result, plugin_status};
#[cfg(feature = "host")]
use upac_types::request::decoder::DecodeRequest;
#[cfg(feature = "host")]
use upac_types::response::decoder::DecodeResponse;

#[cfg(feature = "host")]
use self::error::DecoderError;
#[cfg(feature = "dynamic-plugins")]
use self::manifest::DecoderManifest;

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

#[derive(Debug, Clone, Copy)]
pub struct BuiltinDecoder {
    pub format: &'static str,
    pub extensions: &'static [&'static str],
    pub decode: DecodeFn,
}

#[cfg(feature = "host")]
pub struct Decoder {
    format: String,
    extensions: Vec<String>,
    decode: DecodeFn,

    #[cfg(feature = "dynamic-plugins")]
    _library: Option<Library>,
}

#[cfg(feature = "host")]
impl Decoder {
    #[cfg(feature = "dynamic-plugins")]
    pub fn load(manifest: &DecoderManifest) -> Result<Self, DecoderError> {
        let library = unsafe { Library::new(&manifest.library) }.map_err(|_| DecoderError::Load)?;

        let abi_version: DecodePluginAbiVersionFn = load_symbol!(library, "decode_abi_version");
        let decode: DecodeFn = load_symbol!(library, "decode");

        let got_abi_version = unsafe { abi_version() };
        if got_abi_version != DECODER_ABI_VERSION {
            return Err(DecoderError::AbiMismatch {
                got: got_abi_version,
                expected: DECODER_ABI_VERSION,
            });
        }

        Ok(Decoder {
            format: manifest.format.clone(),
            extensions: manifest.extensions.clone(),
            decode,
            _library: Some(library),
        })
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn matches(&self, file_name: &str) -> bool {
        self.extensions.iter().any(|extension| {
            file_name
                .strip_suffix(extension.as_str())
                .is_some_and(|stem| stem.ends_with('.'))
        })
    }

    pub fn decode(
        &self, package_path: &Path, repo: &Repo, cancel: &CancelToken,
    ) -> Result<DecodeResponse, DecoderError> {
        unsafe extern "C" fn store_object(
            store_ctx: *mut c_void, data: *const u8, len: usize, object_id_out: *mut [u8; 32],
        ) -> i32 {
            fn store(
                store_ctx: *mut c_void, data: *const u8, len: usize, object_id_out: *mut [u8; 32],
            ) -> Result<(), DecodeError> {
                let repo = unsafe { store_ctx.cast::<Repo>().as_ref() }.ok_or(DecodeError::InvalidRequest)?;

                if object_id_out.is_null() || (data.is_null() && len != 0) {
                    return Err(DecodeError::InvalidRequest);
                }

                let bytes = if len == 0 {
                    &[][..]
                } else {
                    unsafe { from_raw_parts(data, len) }
                };

                let object_id = repo.repository().ensure_object(bytes).map_err(|_| DecodeError::Io)?;
                let object_id = <[u8; 32]>::try_from(object_id.as_bytes()).map_err(|_| DecodeError::Io)?;

                unsafe { object_id_out.write(object_id) };

                Ok(())
            }

            plugin_status(store(store_ctx, data, len, object_id_out))
        }

        let checksum = PackageChecksum::compute(package_path, cancel)?;

        let request: CDecodeRequest = DecodeRequest {
            package_path: package_path.to_str().ok_or(DecoderError::InvalidPath)?,
            checksum,
            store_object,
            store_ctx: (repo as *const Repo).cast_mut().cast::<c_void>(),
            cancel_token: cancel,
        }
        .into();

        let mut response = MaybeUninit::<CDecodeResponse>::uninit();

        let status = unsafe { (self.decode)(&request, response.as_mut_ptr()) };
        unsafe { request.free() };

        plugin_result(status).map_err(|error| error.map_or(DecoderError::InvalidResponse, DecoderError::Failed))?;

        let response = unsafe { response.assume_init() };

        Ok(DecodeResponse::try_from(&response)?)
    }
}

#[cfg(feature = "host")]
impl From<&BuiltinDecoder> for Decoder {
    fn from(builtin: &BuiltinDecoder) -> Self {
        Decoder {
            format: builtin.format.to_owned(),
            extensions: builtin
                .extensions
                .iter()
                .map(|extension| (*extension).to_owned())
                .collect(),
            decode: builtin.decode,

            #[cfg(feature = "dynamic-plugins")]
            _library: None,
        }
    }
}

#[cfg(feature = "dynamic-plugins")]
unsafe fn load_symbol<Symbol: Copy>(library: &Library, name: &str) -> Result<Symbol, DecoderError> {
    unsafe { library.get::<Symbol>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| DecoderError::Symbol)
}
