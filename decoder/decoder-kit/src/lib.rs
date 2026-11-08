// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

#[cfg(feature = "plugin")]
use std::collections::BTreeMap;
#[cfg(feature = "plugin")]
use std::ffi::OsStr;
#[cfg(feature = "host")]
use std::fs::File;
#[cfg(feature = "plugin")]
use std::io::{Read, copy};
#[cfg(feature = "host")]
use std::mem::MaybeUninit;
#[cfg(any(feature = "host", feature = "plugin"))]
use std::os::raw::c_void;
#[cfg(feature = "plugin")]
use std::os::unix::ffi::OsStrExt;
#[cfg(feature = "plugin")]
use std::path::Component;
#[cfg(feature = "host")]
use std::path::Path;

#[cfg(feature = "plugin")]
use composefs::MAX_INLINE_CONTENT;
#[cfg(feature = "plugin")]
use composefs::generic_tree::{LeafContent, Stat};
#[cfg(feature = "plugin")]
use composefs::tree::{Leaf, RegularFile};

#[cfg(feature = "dynamic-plugins")]
use libloading::Library;

#[cfg(feature = "plugin")]
use nix::sys::stat::Mode;

#[cfg(feature = "plugin")]
use tar::{Entry, EntryType};

#[cfg(feature = "plugin")]
use zerocopy::FromBytes;
#[cfg(feature = "host")]
use zerocopy::IntoBytes;

#[cfg(feature = "dynamic-plugins")]
use upac_abi::DECODER_ABI_VERSION;
#[cfg(feature = "host")]
use upac_abi::request::CDecodeRequest;
use upac_abi::request::DecodeFn;
#[cfg(feature = "dynamic-plugins")]
use upac_abi::request::DecodePluginAbiVersionFn;
#[cfg(feature = "plugin")]
use upac_abi::request::{CreateFileFn, FinishFileFn};
#[cfg(feature = "host")]
use upac_abi::response::CDecodeResponse;
#[cfg(feature = "plugin")]
use upac_abi::{OWNER_SEPARATOR, OWNER_XATTR};

#[cfg(feature = "plugin")]
use upac_composefs::ObjectID;
#[cfg(feature = "host")]
use upac_composefs::Repo;
#[cfg(feature = "plugin")]
use upac_composefs::tree::entry::TreeEntry;

#[cfg(feature = "host")]
use upac_types::decoder::plugin_status;
#[cfg(any(feature = "host", feature = "plugin"))]
use upac_types::decoder::{DecodeError, ObjectFile, plugin_result};
#[cfg(feature = "host")]
use upac_types::request::DecodeRequest;
#[cfg(feature = "host")]
use upac_types::response::DecodeResponse;
#[cfg(feature = "host")]
use upac_types::{CancelToken, PackageChecksum};

#[cfg(feature = "host")]
use self::error::DecoderError;
#[cfg(feature = "plugin")]
use self::layout::owners::ROOT as ROOT_OWNER;
#[cfg(feature = "plugin")]
use self::layout::tar::XATTR_PREFIX;
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
    create_file: CreateFileFn,
    finish_file: FinishFileFn,
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
    pub fn new(create_file: CreateFileFn, finish_file: FinishFileFn, store_ctx: *mut c_void) -> Self {
        Self {
            create_file,
            finish_file,
            store_ctx,
        }
    }

    pub fn store<Reader: Read>(&self, reader: &mut Reader, size: u64) -> Result<RegularFile<ObjectID>, DecodeError> {
        if size <= MAX_INLINE_CONTENT as u64 {
            let mut content = Vec::new();
            reader.read_to_end(&mut content)?;

            return Ok(RegularFile::Inline(content.into()));
        }

        let mut fd = 0;
        let status = unsafe { (self.create_file)(self.store_ctx, &mut fd) };
        plugin_result::<DecodeError>(status).map_err(|error| error.unwrap_or(DecodeError::Io))?;

        let mut file = unsafe { ObjectFile::from_raw(fd) }.map_err(|_| DecodeError::Io)?;

        if copy(reader, &mut file)? != size {
            return Err(DecodeError::MalformedMetadata);
        }

        let mut object_id = [0u8; 32];
        let status = unsafe { (self.finish_file)(self.store_ctx, i32::from(file), size, &mut object_id) };
        plugin_result::<DecodeError>(status).map_err(|error| error.unwrap_or(DecodeError::Io))?;

        let object_id = ObjectID::read_from_bytes(&object_id).map_err(|_| DecodeError::Io)?;

        Ok(RegularFile::External(object_id, size))
    }

    pub fn write_entry<Reader: Read>(
        &self, dumpfile: &mut String, entry: &mut Entry<'_, Reader>,
    ) -> Result<(), DecodeError> {
        self.write(dumpfile, entry, None)
    }

    pub fn write_entry_with<Reader: Read>(
        &self, dumpfile: &mut String, entry: &mut Entry<'_, Reader>, content: &[u8],
    ) -> Result<(), DecodeError> {
        self.write(dumpfile, entry, Some(content))
    }

    fn write<Reader: Read>(
        &self, dumpfile: &mut String, entry: &mut Entry<'_, Reader>, content: Option<&[u8]>,
    ) -> Result<(), DecodeError> {
        fn read_stat<Reader: Read>(entry: &mut Entry<'_, Reader>) -> Result<Stat, DecodeError> {
            let header = entry.header();

            let st_mode = Mode::from_bits_truncate(header.mode()?).bits();
            let st_uid = u32::try_from(header.uid()?).map_err(|_| DecodeError::MalformedMetadata)?;
            let st_gid = u32::try_from(header.gid()?).map_err(|_| DecodeError::MalformedMetadata)?;
            let st_mtim_sec = i64::try_from(header.mtime()?).map_err(|_| DecodeError::MalformedMetadata)?;

            let user = header
                .username()
                .map_err(|_| DecodeError::InvalidUtf8)?
                .map(str::to_owned);
            let group = header
                .groupname()
                .map_err(|_| DecodeError::InvalidUtf8)?
                .map(str::to_owned);

            let mut xattrs = BTreeMap::new();

            if let (Some(user), Some(group)) = (user, group) {
                insert_owner(&mut xattrs, &user, &group);
            }

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

        let path = entry.path()?.into_owned();
        if path
            .components()
            .all(|component| matches!(component, Component::CurDir | Component::RootDir))
        {
            return Ok(());
        }

        let entry_type = entry.header().entry_type();
        let stat = read_stat(entry)?;

        let content = match entry_type {
            EntryType::Directory => return write_tree_entry(dumpfile, TreeEntry::Directory(path, stat)),
            EntryType::Link => {
                let original = entry.link_name()?.ok_or(DecodeError::MalformedMetadata)?;

                return write_tree_entry(dumpfile, TreeEntry::HardLink(path, original.into_owned()));
            }
            EntryType::Regular | EntryType::Continuous => LeafContent::Regular(match content {
                Some(mut content) => {
                    let size = content.len() as u64;

                    self.store(&mut content, size)?
                }
                None => {
                    let size = entry.size();

                    self.store(entry, size)?
                }
            }),
            EntryType::Symlink => LeafContent::Symlink(
                entry
                    .link_name()?
                    .ok_or(DecodeError::MalformedMetadata)?
                    .as_os_str()
                    .into(),
            ),
            _ => return Ok(()),
        };

        write_tree_entry(dumpfile, TreeEntry::Leaf(path, Leaf { stat, content }))
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
        unsafe extern "C" fn create_file(store_ctx: *mut c_void, fd_out: *mut i32) -> i32 {
            fn create(store_ctx: *mut c_void, fd_out: *mut i32) -> Result<(), DecodeError> {
                let repo = unsafe { store_ctx.cast::<Repo>().as_ref() }.ok_or(DecodeError::InvalidRequest)?;

                if fd_out.is_null() {
                    return Err(DecodeError::InvalidRequest);
                }

                let file = repo.repository().create_object_tmpfile().map_err(|_| DecodeError::Io)?;

                unsafe { fd_out.write(i32::from(ObjectFile::from(File::from(file)))) };

                Ok(())
            }

            plugin_status(create(store_ctx, fd_out))
        }

        unsafe extern "C" fn finish_file(
            store_ctx: *mut c_void, fd: i32, size: u64, object_id_out: *mut [u8; 32],
        ) -> i32 {
            fn finish(
                store_ctx: *mut c_void, fd: i32, size: u64, object_id_out: *mut [u8; 32],
            ) -> Result<(), DecodeError> {
                let file = unsafe { ObjectFile::from_raw(fd) }
                    .map_err(|_| DecodeError::InvalidRequest)?
                    .into_file();

                let repo = unsafe { store_ctx.cast::<Repo>().as_ref() }.ok_or(DecodeError::InvalidRequest)?;

                if object_id_out.is_null() {
                    return Err(DecodeError::InvalidRequest);
                }

                let (object_id, _method) = repo
                    .repository()
                    .finalize_object_tmpfile(file, size)
                    .map_err(|_| DecodeError::Io)?;
                let object_id = <[u8; 32]>::try_from(object_id.as_bytes()).map_err(|_| DecodeError::Io)?;

                unsafe { object_id_out.write(object_id) };

                Ok(())
            }

            plugin_status(finish(store_ctx, fd, size, object_id_out))
        }

        let checksum = PackageChecksum::compute(package_path, cancel)?;

        let request: CDecodeRequest = DecodeRequest {
            package_path: package_path.to_str().ok_or(DecoderError::InvalidPath)?,
            checksum,
            create_file,
            finish_file,
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

#[cfg(feature = "plugin")]
pub fn insert_owner(xattrs: &mut BTreeMap<Box<OsStr>, Box<[u8]>>, user: &str, group: &str) {
    if user.is_empty() || group.is_empty() || (user == ROOT_OWNER && group == ROOT_OWNER) {
        return;
    }

    xattrs.insert(
        Box::from(OsStr::new(OWNER_XATTR)),
        Box::from(format!("{user}{OWNER_SEPARATOR}{group}").as_bytes()),
    );
}

#[cfg(feature = "plugin")]
pub fn write_tree_entry(dumpfile: &mut String, tree_entry: TreeEntry) -> Result<(), DecodeError> {
    tree_entry.write(dumpfile).map_err(|_| DecodeError::MalformedMetadata)
}

#[cfg(feature = "dynamic-plugins")]
unsafe fn load_symbol<Symbol: Copy>(library: &Library, name: &str) -> Result<Symbol, DecoderError> {
    unsafe { library.get::<Symbol>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| DecoderError::Symbol)
}
