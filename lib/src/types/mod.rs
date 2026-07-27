use upac_abi::error::ErrorKind;
use upac_abi::package::{CPackageMeta, CUnpackedPackage, CVersion};
use upac_abi::types::CBorrowed;
use upac_macro::RedbCodec;

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
#[derive(Debug, Clone, PartialEq, Eq, RedbCodec)]
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

        let pre = if version.pre.ptr.is_null() {
            None
        } else {
            let pre: &str = (&version.pre).try_into()?;
            Some(pre.to_owned())
        };

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

#[derive(Debug, Clone, RedbCodec)]
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

        let arch_sub = if meta.arch_sub.ptr.is_null() {
            None
        } else {
            let arch_sub: &str = (&meta.arch_sub).try_into()?;
            Some(arch_sub.to_owned())
        };

        let license = if meta.license.ptr.is_null() {
            None
        } else {
            let license: &str = (&meta.license).try_into()?;
            Some(license.to_owned())
        };

        let url = if meta.url.ptr.is_null() {
            None
        } else {
            let url: &str = (&meta.url).try_into()?;
            Some(url.to_owned())
        };

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
