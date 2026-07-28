#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDomain {
    Uninstall,
    Install,
    Rollback,
    Commit,
    Files,
    Update,
    ListPackages,
    ListCommit,
    DiffFiles,
    DiffPackages,
    Diff,
    SearchMeta,
    SearchFiles,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unexpected,
    OutOfMemory,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    InvalidPath,
    NoSpaceLeft,
    Cancelled,
    ReadFailed,
    WriteFailed,
    NotInitialized,
    AbiMismatch,
    InvalidEntry,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CError {
    pub domain: ErrorDomain,
    pub state: u32,
    pub error: ErrorKind,
}

pub trait CommandState: Copy {
    const DOMAIN: ErrorDomain;
    const VALIDATION: Self;

    fn as_u32(self) -> u32;
}
