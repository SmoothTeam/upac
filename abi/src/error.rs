#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDomain {
    Uninstall,
    Install,
    Rollback,
    Commit,
    Files,
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
    MaxRetriesExceeded,
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
