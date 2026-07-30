use upac_abi::error::{CommandState, ErrorDomain};
use upac_macro::FromStageIndex;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum InstallStateId {
    Preparation = 0,
    Transaction = 1,
    Merge = 2,
    Checkout = 3,
    Swap = 4,
    Done = 5,
    Setup = 6,
}

impl CommandState for InstallStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Install;
    const VALIDATION: Self = InstallStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum UninstallStateId {
    Preparation = 0,
    Build = 1,
    Commit = 2,
    ConfigMerge = 3,
    PrepareBoot = 4,
    BootOption = 5,
    Done = 6,
    Setup = 7,
}

impl CommandState for UninstallStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Uninstall;
    const VALIDATION: Self = UninstallStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum RollbackStateId {
    Merge = 0,
    Checkout = 1,
    Swap = 2,
    Done = 3,
    Setup = 4,
}

impl CommandState for RollbackStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Rollback;
    const VALIDATION: Self = RollbackStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum FilesStateId {
    Transaction = 0,
    Checkout = 1,
    Swap = 2,
    Done = 3,
    Setup = 4,
}

impl CommandState for FilesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Files;
    const VALIDATION: Self = FilesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPackagesStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListPackagesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListPackages;
    const VALIDATION: Self = ListPackagesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListCommitStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListCommitStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListCommit;
    const VALIDATION: Self = ListCommitStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListPrefixStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListPrefixStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListPrefix;
    const VALIDATION: Self = ListPrefixStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListHistoryStateId {
    Fetching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for ListHistoryStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::ListHistory;
    const VALIDATION: Self = ListHistoryStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFilesStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffFilesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::DiffFiles;
    const VALIDATION: Self = DiffFilesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffPackagesStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffPackagesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::DiffPackages;
    const VALIDATION: Self = DiffPackagesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStateId {
    Preparing = 0,
    Comparing = 1,
    Done = 2,
    Setup = 3,
}

impl CommandState for DiffStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Diff;
    const VALIDATION: Self = DiffStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum UpdateStateId {
    Preparation = 0,
    Transaction = 1,
    Merge = 2,
    Checkout = 3,
    Swap = 4,
    Done = 5,
    Setup = 6,
}

impl CommandState for UpdateStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Update;
    const VALIDATION: Self = UpdateStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMetaStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for SearchMetaStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::SearchMeta;
    const VALIDATION: Self = SearchMetaStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFilesStateId {
    Searching = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for SearchFilesStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::SearchFiles;
    const VALIDATION: Self = SearchFilesStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStageIndex)]
pub enum CommitStateId {
    Transaction = 0,
    Done = 1,
    Setup = 2,
}

impl CommandState for CommitStateId {
    const DOMAIN: ErrorDomain = ErrorDomain::Commit;
    const VALIDATION: Self = CommitStateId::Setup;

    fn as_u32(self) -> u32 {
        self as u32
    }
}
