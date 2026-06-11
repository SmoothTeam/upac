// ── Errors ───────────────────────────────────────────────────────────────────
pub const DiffError = error{
    RepoOpenFailed,
    CommitNotFound,
    DiffFailed,
    AllocFailed,
    OutOfMemory,
    FileNotFound,
    PathNotFound,
    CheckSpaceFailed,
    NotEnoughSpace,
    CheckoutFailed,
    ReadDatabaseFailed,
    AccessDenied,
    Cancelled,
};
