use std::mem::size_of;

use upac_abi::DiffKind;
use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageMeta, CUnpackedPackage, CVersion};
use upac_abi::response::{CCommitEntry, CDiffFileEntry, CDiffPackageEntry, CHistoryEntry, CPrefixEntry, CSearchFileEntry};
use upac_abi::types::{CBorrowed, COwned, CSlice, CVec};
use upac_macro::{RedbCodec, RustToC};

include!(concat!(env!("OUT_DIR"), "/layout.rs"));

pub mod errors;
pub mod lock;
pub mod states;

macro_rules! as_str_method {
    ($name:ty) => {
        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

// ── Version ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, RedbCodec, RustToC)]
pub struct Version {
    pub epoch: u32,
    pub parts: Vec<u32>,
    pub pre: Option<String>,
    pub release: u32,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            epoch: 0,
            parts: Vec::new(),
            pre: None,
            release: 1,
        }
    }
}

impl TryFrom<&CVersion> for Version {
    type Error = ErrorKind;

    fn try_from(version: &CVersion) -> Result<Self, ErrorKind> {
        unsafe { version.validate()? };

        let pre = Option::<&str>::try_from(&version.pre)?.map(str::to_owned);

        Ok(Version {
            epoch: version.epoch,
            parts: unsafe { version.parts.as_borrowed() }.to_vec(),
            pre,
            release: version.release,
        })
    }
}

// ── Package ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PackageTemp {
    pub meta: PackageMeta,
    pub temp_package_path: String,
}

impl TryFrom<&CUnpackedPackage> for PackageTemp {
    type Error = ErrorKind;

    fn try_from(package: &CUnpackedPackage) -> Result<Self, ErrorKind> {
        unsafe { package.validate()? };

        let temp_package_path: &str = (&package.temp_path).try_into()?;

        Ok(PackageTemp {
            meta: PackageMeta::try_from(&package.meta)?,
            temp_package_path: temp_package_path.to_owned(),
        })
    }
}

#[derive(Debug, Clone, RedbCodec, RustToC)]
pub struct PackageMeta {
    pub name: String,
    pub version: Version,
    pub arch: String,
    pub arch_sub: Option<String>,
    pub maintainer: String,
    pub description: String,
    pub license: Option<String>,
    pub url: Option<String>,
    pub sha256: [u8; 32],
    pub installed_size: u64,
}

impl TryFrom<&CPackageMeta> for PackageMeta {
    type Error = ErrorKind;

    fn try_from(meta: &CPackageMeta) -> Result<Self, ErrorKind> {
        unsafe { meta.validate()? };

        let name: &str = (&meta.name).try_into()?;
        let arch: &str = (&meta.arch).try_into()?;
        let maintainer: &str = (&meta.maintainer).try_into()?;
        let description: &str = (&meta.description).try_into()?;

        let arch_sub = Option::<&str>::try_from(&meta.arch_sub)?.map(str::to_owned);
        let license = Option::<&str>::try_from(&meta.license)?.map(str::to_owned);
        let url = Option::<&str>::try_from(&meta.url)?.map(str::to_owned);

        Ok(PackageMeta {
            name: name.to_owned(),
            version: Version::try_from(&meta.version)?,
            arch: arch.to_owned(),
            arch_sub,
            maintainer: maintainer.to_owned(),
            description: description.to_owned(),
            license,
            url,
            sha256: meta.sha256,
            installed_size: meta.installed_size,
        })
    }
}

// ── PackageEntry ────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub arch: String,
    pub arch_sub: Option<String>,
}

// ── FileEntry ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RedbCodec)]
pub struct FileEntry {
    pub path: String,
    pub is_user: bool,
}

// ── SearchFileEntry ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct SearchFileEntry {
    pub path: String,
    pub package_name: String,
    pub is_user: bool,
}

// ── PrefixEntry ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct PrefixEntry {
    pub prefix_digest: String,

    pub subject: String,
    pub message: Option<String>,

    pub timestamp: u64,

    pub working_config: Option<String>,
}

// ── CommitEntry ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct CommitEntry {
    pub config_digest: String,

    pub subject: String,
    pub message: Option<String>,
}

// ── HistoryEntry ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct HistoryEntry {
    pub prefix_digest: String,

    pub subject: String,
    pub message: Option<String>,

    pub timestamp: u64,

    pub working_config: Option<String>,
    pub config_history: Vec<CommitEntry>,
}

// ── DiffFileEntry ───────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffFileEntry {
    pub path: String,
    pub kind: DiffKind,
    pub package_name: String,
    pub is_user: bool,
}

// ── DiffPackageEntry ────────────────────────────────────────────────────────
#[derive(Debug, Clone, RustToC)]
pub struct DiffPackageEntry {
    pub name: String,
    pub kind: DiffKind,
    pub version: Version,
}

pub struct Targets(pub Vec<PackageEntry>);

impl Targets {
    pub fn entries(&self) -> &[PackageEntry] {
        &self.0
    }
}

pub struct TmpPath(pub String);

as_str_method!(TmpPath);

pub struct Branch(pub String);

as_str_method!(Branch);
