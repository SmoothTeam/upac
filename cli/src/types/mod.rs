pub mod backend;
pub mod commit;
pub mod errors;
pub mod package;

pub use crate::ffi::DiffKind;

pub const EXPECTED_ABI_VERSION: u32 = 2;

pub struct PackageDiffEntry {
    pub name: String,
    pub kind: DiffKind,
}

pub struct FileDiffEntry {
    pub path: String,
    pub kind: DiffKind,
    pub package_name: String,
}
