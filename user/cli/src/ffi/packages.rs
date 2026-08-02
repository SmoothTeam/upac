use std::ffi::CString;

use super::ctypes::{CDiffKind, CSlice, CVersion};
use super::Validate;

// ── CPackage ──────────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPackage {
    struct_size: usize,
    pub meta: *mut CPackageMeta,
    pub temp_path: CSlice,
}

impl CPackage {
    pub fn new(meta: *mut CPackageMeta, temp_path: CSlice) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            meta,
            temp_path,
        }
    }
}

// ── CPackageMeta ──────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPackageMeta {
    struct_size: usize,
    pub name: CSlice,
    pub version: CVersion,
    pub arch: CSlice,
    pub arch_sub: CSlice,
    pub maintainer: CSlice,
    pub description: CSlice,
    pub license: CSlice,
    pub url: CSlice,
    pub sha256: [u8; 32],
    pub installed_size: u64,
}

// ── CPackageInfo ──────────────────────────────────────────────────────────────
#[repr(C)]
pub struct CPackageInfo {
    struct_size: usize,
    pub name: CSlice,
    pub arch: CSlice,
    pub arch_sub: CSlice,
}

impl CPackageInfo {
    pub fn new(name: &CString, arch: &CString, arch_sub: Option<&CString>) -> Self {
        Self {
            struct_size: size_of::<Self>(),
            name: CSlice::from_cstring(name),
            arch: CSlice::from_cstring(arch),
            arch_sub: arch_sub.map(CSlice::from_cstring).unwrap_or(CSlice::empty()),
        }
    }
}

// ── Diff entry types ──────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CDiffPackageEntry {
    struct_size: usize,
    pub name: CSlice,
    pub kind: CDiffKind,
    pub version: CVersion,
}

impl Validate for CDiffPackageEntry {
    fn validate(&self) -> anyhow::Result<()> {
        if self.struct_size != size_of::<Self>() {
            return Err(anyhow::anyhow!("CDiffPackageEntry: abi mismatch"));
        }
        self.name.validate()?;
        Ok(())
    }
}
