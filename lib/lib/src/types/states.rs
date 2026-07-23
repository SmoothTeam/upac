#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Added = 0,
    Removed = 1,
    Modified = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStateId {
    Verifying = 0,
    Preparation = 1,
    Transaction = 2,
    Merge = 3,
    Checkout = 4,
    Swap = 5,
    Done = 6,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallStateId {
    Preparation = 0,
    Build = 1,
    Commit = 2,
    ConfigMerge = 3,
    PrepareBoot = 4,
    BootOption = 5,
    Done = 6,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackStateId {
    Verifying = 0,
    Merge = 1,
    Checkout = 2,
    Swap = 3,
    Done = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesStateId {
    Verifying = 0,
    Transaction = 1,
    Checkout = 2,
    Swap = 3,
    Done = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPackagesStateId {
    Verifying = 0,
    Fetching = 1,
    Done = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListCommitStateId {
    Verifying = 0,
    Fetching = 1,
    Done = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStateId {
    Verifying = 0,
    Preparing = 1,
    Comparing = 2,
    Done = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitStateId {
    Verifying = 0,
    Setup = 1,
    Commit = 2,
    Done = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStateId {
    Verifying = 0,
    Preparation = 1,
    Transaction = 2,
    Merge = 3,
    Checkout = 4,
    Swap = 5,
    Done = 6,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMetaStateId {
    Verifying = 0,
    Searching = 1,
    Done = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFilesStateId {
    Verifying = 0,
    Searching = 1,
    Done = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitStateId {
    Verifying = 0,
    Transaction = 1,
    Done = 2,
}
