// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

#[cfg(feature = "plugin")]
use std::collections::BTreeMap;
#[cfg(feature = "plugin")]
use std::ffi::OsStr;
#[cfg(feature = "plugin")]
use std::io::Read;
#[cfg(feature = "host")]
use std::mem::MaybeUninit;
#[cfg(any(feature = "host", feature = "plugin"))]
use std::os::raw::c_void;
#[cfg(feature = "plugin")]
use std::os::unix::ffi::OsStrExt;
#[cfg(any(feature = "host", feature = "plugin"))]
use std::path::Path;
#[cfg(feature = "plugin")]
use std::path::{Component, PathBuf};
#[cfg(feature = "host")]
use std::slice::from_raw_parts;

#[cfg(feature = "plugin")]
use composefs::MAX_INLINE_CONTENT;
#[cfg(feature = "plugin")]
use composefs::generic_tree::{LeafContent, Stat};
#[cfg(feature = "plugin")]
use composefs::tree::RegularFile;

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "plugin")]
use tar::{Entry, EntryType};

#[cfg(feature = "plugin")]
use zerocopy::FromBytes;
#[cfg(feature = "host")]
use zerocopy::IntoBytes;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::DECODER_ABI_VERSION;
use upac_abi::plugin::DecodeFn;
#[cfg(feature = "dynamic-plugins")]
use upac_abi::plugin::DecodePluginAbiVersionFn;
#[cfg(feature = "plugin")]
use upac_abi::plugin::StoreObjectFn;
#[cfg(feature = "host")]
use upac_abi::request::decoder::CDecodeRequest;
#[cfg(feature = "host")]
use upac_abi::response::decoder::CDecodeResponse;

#[cfg(feature = "plugin")]
use upac_composefs::ObjectID;
#[cfg(feature = "host")]
use upac_composefs::Repo;
#[cfg(feature = "plugin")]
use upac_composefs::error::RepoError;
#[cfg(feature = "plugin")]
use upac_composefs::tree::builder::TreeBuilder;

#[cfg(feature = "host")]
use upac_types::CancelToken;
#[cfg(any(feature = "host", feature = "plugin"))]
use upac_types::decoder::DecodeError;
#[cfg(feature = "host")]
use upac_types::package::PackageChecksum;
#[cfg(any(feature = "host", feature = "plugin"))]
use upac_types::plugin::plugin_result;
#[cfg(feature = "host")]
use upac_types::plugin::plugin_status;
#[cfg(feature = "host")]
use upac_types::request::decoder::DecodeRequest;
#[cfg(feature = "host")]
use upac_types::response::decoder::DecodeResponse;

#[cfg(feature = "host")]
use self::error::DecoderError;
#[cfg(feature = "plugin")]
use self::layout::tar::{PERMISSION_MASK, XATTR_PREFIX};
#[cfg(feature = "dynamic-plugins")]
use self::manifest::DecoderManifest;

#[cfg(feature = "host")]
pub mod error;
#[cfg(feature = "plugin")]
mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
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

#[cfg(feature = "plugin")]
pub struct ObjectStore {
    store_object: StoreObjectFn,
    store_ctx: *mut c_void,
}

#[cfg(feature = "host")]
pub struct Decoder {
    format: String,
    extensions: Vec<String>,
    decode: DecodeFn,

    #[cfg(feature = "dynamic-plugins")]
    _library: Option<Library>,
}

#[cfg(feature = "plugin")]
impl ObjectStore {
    pub fn new(store_object: StoreObjectFn, store_ctx: *mut c_void) -> Self {
        Self {
            store_object,
            store_ctx,
        }
    }

    pub fn store(&self, data: &[u8]) -> Result<RegularFile<ObjectID>, DecodeError> {
        if data.len() <= MAX_INLINE_CONTENT {
            return Ok(RegularFile::Inline(data.into()));
        }

        let mut object_id = [0u8; 32];
        let status = unsafe { (self.store_object)(self.store_ctx, data.as_ptr(), data.len(), &mut object_id) };
        plugin_result::<DecodeError>(status).map_err(|error| error.unwrap_or(DecodeError::Io))?;

        let object_id = ObjectID::read_from_bytes(&object_id).map_err(|_| DecodeError::Io)?;

        Ok(RegularFile::External(object_id, data.len() as u64))
    }

    pub fn insert_entry<Reader: Read>(
        &self, builder: &mut TreeBuilder, entry: &mut Entry<'_, Reader>,
    ) -> Result<(), DecodeError> {
        fn normalize(path: &Path) -> PathBuf {
            path.components()
                .filter(|component| matches!(component, Component::Normal(_)))
                .collect()
        }

        fn read_stat<Reader: Read>(entry: &mut Entry<'_, Reader>) -> Result<Stat, DecodeError> {
            let header = entry.header();

            let st_mode = header.mode()? & PERMISSION_MASK;
            let st_uid = u32::try_from(header.uid()?).map_err(|_| DecodeError::MalformedMetadata)?;
            let st_gid = u32::try_from(header.gid()?).map_err(|_| DecodeError::MalformedMetadata)?;
            let st_mtim_sec = i64::try_from(header.mtime()?).map_err(|_| DecodeError::MalformedMetadata)?;

            let mut xattrs = BTreeMap::new();

            if let Some(extensions) = entry.pax_extensions()? {
                for extension in extensions {
                    let extension = extension?;

                    if let Some(name) = extension.key_bytes().strip_prefix(XATTR_PREFIX.as_bytes()) {
                        xattrs.insert(Box::from(OsStr::from_bytes(name)), Box::from(extension.value_bytes()));
                    }
                }
            }

            Ok(Stat {
                st_mode,
                st_uid,
                st_gid,
                st_mtim_sec,
                st_mtim_nsec: 0,
                xattrs,
            })
        }

        let path = normalize(&entry.path()?);
        if path.as_os_str().is_empty() {
            return Ok(());
        }

        let entry_type = entry.header().entry_type();
        let stat = read_stat(entry)?;

        let inserted = match entry_type {
            EntryType::Directory => match builder.try_insert_dir(&path, stat.clone()) {
                Err(RepoError::AlreadyExists) => builder.update_metadata(&path, stat),
                inserted => inserted,
            },
            EntryType::Regular | EntryType::Continuous => {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;

                builder.insert(&path, LeafContent::Regular(self.store(&data)?), stat)
            }
            EntryType::Symlink => {
                let target = entry.link_name()?.ok_or(DecodeError::MalformedMetadata)?;

                builder.insert(&path, LeafContent::Symlink(target.as_os_str().into()), stat)
            }
            EntryType::Link => {
                let original = normalize(&entry.link_name()?.ok_or(DecodeError::MalformedMetadata)?);

                builder.insert_hard_link(&path, &original)
            }
            _ => Ok(()),
        };

        inserted.map_err(|_| DecodeError::MalformedMetadata)
    }
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
