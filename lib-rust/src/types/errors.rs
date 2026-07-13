use nix::errno::Errno;

use crate::types::deploy::SysrootError;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Ok = 0,

    Unexpected = 1,
    OutOfMemory = 2,
    FileNotFound = 3,
    PermissionDenied = 4,
    InvalidPath = 5,
    NoSpaceLeft = 6,
    AbiMismatch = 7,

    TreadError = 9,
    LockWouldBlock = 10,
    AlloczFailed = 11,
    Cancelled = 12,
    MaxRetriesExceeded = 13,
    ReadFailed = 14,
    WriteFailed = 15,
    DiffFailed = 16,
    ListFailed = 17,

    DbMissingField = 30,
    DbMissingSection = 31,
    DbInvalidEntry = 32,
    DbParseError = 33,
    DbWriteDatabaseFailed = 34,
    DbMalformedMeta = 35,
    DbMalformedFiles = 36,
    IdxMalformedEntry = 37,
    DbReadDatabaseFailed = 38,
    DbPackageNotFound = 39,
    DbPackageAlreadyExists = 40,
    DbPackageFilesExist = 41,
    DbArchitectureNotFound = 42,
    DbNotInitialized = 43,

    InstallAlreadyInstalled = 50,
    InstallPackagePathNotFound = 51,
    InstallCollectFileChecksumFailed = 52,
    InstallCheckoutFailed = 53,
    InstallCancelled = 54,
    InstallMaxRetriesExceeded = 55,
    InstallCheckSpaceFailed = 56,
    InstallWriteFilesFailed = 57,
    InstallWriteConfigFailed = 58,

    UpdatePackageNotFound = 60,
    UpdateCollectFileChecksumFailed = 61,
    UpdateCheckoutFailed = 62,
    UpdateCancelled = 63,
    UpdateCheckSpaceFailed = 64,
    UpdateWriteFilesFailed = 65,
    UpdateWriteConfigFailed = 66,

    UninstallNotFound = 70,
    UninstallFailed = 71,
    UninstallFileMapCorrupted = 72,
    UninstallStagingNotCleaned = 73,

    OstreeRepoOpenFailed = 90,
    OstreeRepoTransactionFailed = 91,
    OstreeCommit = 92,
    OstreeRollback = 93,
    OstreeNoParent = 94,
    OstreeStagingFailed = 95,
    OstreeSwapFailed = 96,
    OstreeCommitNotFound = 97,
    OstreeCleanupFailed = 98,
    OstreeRepoWriteFailed = 99,
    OstreeMtreeInsertFailed = 100,
    OstreeMtreeWriteFailed = 101,
    OstreeCommitWriteFailed = 102,

    AlreadyInitialized = 110,
    CreateDirFailed = 111,
    NotADirectory = 112,
    OstreeInitFailed = 113,
    DirectoryNotEmpty = 114,
    InitSymlinkFailed = 115,

    FileChecksumFailed = 120,
    FileAlreadyExists = 121,

    SysrootMountInfoUnavailable = 130,
    SysrootRootDeviceNotFound = 131,
    SysrootCacheUnavailable = 132,
    SysrootUuidNotFound = 133,
    SysrootCanonicalDeviceNotFound = 134,
    SysrootMountFailed = 135,
    SysrootDirUnavailable = 136,
    SysrootDeploysDirNotFound = 137,
    SysrootRepoDirNotFound = 138,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    PackageNotFound,
    PackageAlreadyExists,
    PackageFilesExist,
    ArchitectureNotFound,
    DatabaseNotInitialized,
    AllocZFailed,
    AccessDenied,
    WriteError,
    ReadError,
}

impl From<DatabaseError> for ErrorCode {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::PackageNotFound => ErrorCode::DbPackageNotFound,
            DatabaseError::PackageAlreadyExists => ErrorCode::DbPackageAlreadyExists,
            DatabaseError::PackageFilesExist => ErrorCode::DbPackageFilesExist,
            DatabaseError::ArchitectureNotFound => ErrorCode::DbArchitectureNotFound,
            DatabaseError::DatabaseNotInitialized => ErrorCode::DbNotInitialized,
            DatabaseError::AllocZFailed => ErrorCode::AlloczFailed,
            DatabaseError::AccessDenied => ErrorCode::PermissionDenied,
            DatabaseError::WriteError => ErrorCode::DbWriteDatabaseFailed,
            DatabaseError::ReadError => ErrorCode::DbReadDatabaseFailed,
        }
    }
}

impl From<SysrootError> for ErrorCode {
    fn from(error: SysrootError) -> Self {
        match error {
            SysrootError::MountInfoUnavailable => ErrorCode::SysrootMountInfoUnavailable,
            SysrootError::RootDeviceNotFound => ErrorCode::SysrootRootDeviceNotFound,
            SysrootError::CacheUnavailable => ErrorCode::SysrootCacheUnavailable,
            SysrootError::UuidNotFound => ErrorCode::SysrootUuidNotFound,
            SysrootError::CanonicalDeviceNotFound => ErrorCode::SysrootCanonicalDeviceNotFound,
            SysrootError::SysrootDirUnavailable => ErrorCode::SysrootDirUnavailable,
            SysrootError::DeploysDirNotFound => ErrorCode::SysrootDeploysDirNotFound,
            SysrootError::RepoDirNotFound => ErrorCode::SysrootRepoDirNotFound,
            SysrootError::System(_) => ErrorCode::SysrootMountFailed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonError {
    OutOfMemory,
    Cancelled,
    AccessDenied,
    MaxRetriesExceeded,
    RepoOpenFailed,
    RepoTransactionFailed,
    MtreeWriteFailed,
    MtreeInsertFailed,
    CommitWriteFailed,
    Database(DatabaseError),
    Sysroot(SysrootError),
}

impl From<CommonError> for ErrorCode {
    fn from(error: CommonError) -> Self {
        match error {
            CommonError::OutOfMemory => ErrorCode::OutOfMemory,
            CommonError::Cancelled => ErrorCode::Cancelled,
            CommonError::AccessDenied => ErrorCode::PermissionDenied,
            CommonError::MaxRetriesExceeded => ErrorCode::MaxRetriesExceeded,
            CommonError::RepoOpenFailed => ErrorCode::OstreeRepoOpenFailed,
            CommonError::RepoTransactionFailed => ErrorCode::OstreeRepoTransactionFailed,
            CommonError::MtreeWriteFailed => ErrorCode::OstreeMtreeWriteFailed,
            CommonError::MtreeInsertFailed => ErrorCode::OstreeMtreeInsertFailed,
            CommonError::CommitWriteFailed => ErrorCode::OstreeCommitWriteFailed,
            CommonError::Database(database_error) => database_error.into(),
            CommonError::Sysroot(sysroot_error) => sysroot_error.into(),
        }
    }
}

impl From<DatabaseError> for CommonError {
    fn from(error: DatabaseError) -> Self {
        CommonError::Database(error)
    }
}

impl From<SysrootError> for CommonError {
    fn from(error: SysrootError) -> Self {
        CommonError::Sysroot(error)
    }
}

pub enum LockError {
    Busy,
    ReadOnly,
    Denied,
    PathMissing,
    Unexpected(Errno),
}

impl From<Errno> for LockError {
    fn from(errno: Errno) -> Self {
        match errno {
            Errno::EADDRINUSE => LockError::Busy,
            Errno::EROFS => LockError::ReadOnly,
            Errno::EPERM | Errno::EACCES => LockError::Denied,
            Errno::ENOENT => LockError::PathMissing,
            other => LockError::Unexpected(other),
        }
    }
}

impl From<LockError> for ErrorCode {
    fn from(error: LockError) -> Self {
        match error {
            LockError::Busy => ErrorCode::LockWouldBlock,
            LockError::ReadOnly | LockError::Denied => ErrorCode::PermissionDenied,
            LockError::PathMissing => ErrorCode::InvalidPath,
            LockError::Unexpected(_) => ErrorCode::Unexpected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallError {
    PackageNotFound,
    UninstallFailed,
    FileMapCorrupted,
    StagingNotCleaned,
    CheckoutFailed,
    ReadDatabaseFailed,
    WriteDatabaseFailed,
    Common(CommonError),
}

impl From<CommonError> for UninstallError {
    fn from(error: CommonError) -> Self {
        UninstallError::Common(error)
    }
}

impl From<DatabaseError> for UninstallError {
    fn from(error: DatabaseError) -> Self {
        UninstallError::Common(CommonError::Database(error))
    }
}

impl From<SysrootError> for UninstallError {
    fn from(error: SysrootError) -> Self {
        UninstallError::Common(CommonError::Sysroot(error))
    }
}

impl From<UninstallError> for ErrorCode {
    fn from(error: UninstallError) -> Self {
        match error {
            UninstallError::PackageNotFound => ErrorCode::UninstallNotFound,
            UninstallError::UninstallFailed => ErrorCode::UninstallFailed,
            UninstallError::FileMapCorrupted => ErrorCode::UninstallFileMapCorrupted,
            UninstallError::StagingNotCleaned => ErrorCode::UninstallStagingNotCleaned,
            UninstallError::CheckoutFailed => ErrorCode::InstallCheckoutFailed,
            UninstallError::ReadDatabaseFailed => ErrorCode::DbReadDatabaseFailed,
            UninstallError::WriteDatabaseFailed => ErrorCode::DbWriteDatabaseFailed,
            UninstallError::Common(common_error) => common_error.into(),
        }
    }
}

pub fn to_code<E: Into<ErrorCode>>(result: Result<(), E>) -> i32 {
    match result {
        Ok(()) => ErrorCode::Ok as i32,
        Err(error) => error.into() as i32,
    }
}
