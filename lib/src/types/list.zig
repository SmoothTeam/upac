// ── Errors ────────────────────────────────────────────────────────────────────
pub const ListError = error{
    PathNotFound,
    RepoOpenFailed,
    DatabaseNotFound,
    FetchFailed,
    AllocFailed,
    OutOfMemory,
    CommitFailed,
    Cancelled,
};
