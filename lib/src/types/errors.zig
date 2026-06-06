pub const Operation = enum { install, uninstall, update, rollback, init, diff, list, files, search, commit };

// A listing of all possible return codes used to signal success or specific runtime errors
pub const ErrorCode = enum(i32) {
    ok = 0,

    // --- General Errors (0 - 29) ---
    unexpected = 1,
    out_of_memory = 2,
    file_not_found = 3,
    permission_denied = 4,
    invalid_path = 5,
    no_space_left = 6,
    abi_mismatch = 7,

    tread_error = 9,
    lock_would_block = 10,
    allocz_failed = 11,
    cancelled = 12,
    max_retries_exceeded = 13,
    read_failed = 14,
    write_failed = 15,
    diff_failed = 16,
    list_failed = 17,

    // --- Database & Index Errors (30 - 49) ---
    db_missing_field = 30,
    db_missing_section = 31,
    db_invalid_entry = 32,
    db_parse_error = 33,
    db_write_database_failed = 34,
    db_malformed_meta = 35,
    db_malformed_files = 36,
    idx_malformed_entry = 37,

    // --- Installer Errors (50 - 69) ---
    install_already_installed = 50,
    install_package_path_not_found = 51,
    install_collect_file_checksum_failed = 52,
    install_checkout_failed = 53,
    install_cancelled = 54,
    install_max_retries_exceeded = 55,
    install_check_space_failed = 56,
    install_write_files_failed = 57,
    install_write_config_failed = 58,

    // --- Updater Errors (60 - 69) ---
    update_package_not_found = 60,
    update_collect_file_checksum_failed = 61,
    update_checkout_failed = 62,
    update_cancelled = 63,
    update_check_space_failed = 64,
    update_write_files_failed = 65,
    update_write_config_failed = 66,

    // --- Uninstaller Errors (70 - 89) ---
    uninstall_not_found = 70,
    uninstall_failed = 71,
    uninstall_file_map_corrupted = 72,
    uninstall_staging_not_cleaned = 73,

    // --- OSTree, Repo & FSM Errors (90 - 109) ---
    ostree_repo_open_failed = 90,
    ostree_repo_transaction_failed = 91,
    ostree_commit = 92,
    ostree_rollback = 93,
    ostree_no_parent = 94,
    ostree_staging_failed = 95,
    ostree_swap_failed = 96,
    ostree_commit_not_found = 97,
    ostree_cleanup_failed = 98,
    ostree_repo_write_failed = 99,
    ostree_mtree_insert_failed = 100,

    // --- Init Errors (110 - 119) ---
    already_initialized = 110,
    create_dir_failed = 111,
    not_a_directory = 112,
    ostree_init_failed = 113,
    directory_not_empty = 114,
    init_symlink_failed = 115,

    // --- File Checksum/FSM Errors (120+) ---
    file_checksum_failed = 120,
    file_already_exists = 121,
};

// A mapper function that translates internal Zig errors (anyerror) into ErrorCode values understandable by the external interface
pub fn fromError(err: anyerror, operation: Operation) ErrorCode {
    const specific: ?ErrorCode = switch (operation) {
        .init => switch (err) {
            error.RootNotFound => .invalid_path,
            error.AlreadyInitialized => .already_initialized,
            error.CreateDirFailed => .create_dir_failed,
            error.NotADirectory => .not_a_directory,
            error.DirectoryNotEmpty => .directory_not_empty,
            error.OstreeInitFailed => .ostree_init_failed,
            error.PrefixNotFound => .invalid_path,
            error.SymlinkFailed => .init_symlink_failed,
            error.DatabaseInitFailed => .db_write_database_failed,
            else => null,
        },
        .install => switch (err) {
            error.AlreadyInstalled => .install_already_installed,
            error.PackagePathNotFound => .install_package_path_not_found,
            error.CollectFileChecksumsFailed => .install_collect_file_checksum_failed,
            error.CheckoutFailed => .install_checkout_failed,
            error.Cancelled => .install_cancelled,
            error.MaxRetriesExceeded => .install_max_retries_exceeded,
            error.CheckSpaceFailed => .install_check_space_failed,
            error.WriteFilesFailed => .install_write_files_failed,
            error.WriteConfigFailed => .install_write_config_failed,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            else => null,
        },
        .update => switch (err) {
            error.PackageNotFound => .update_package_not_found,
            error.CollectFileChecksumsFailed => .update_collect_file_checksum_failed,
            error.CheckoutFailed => .update_checkout_failed,
            error.Cancelled => .update_cancelled,
            error.CheckSpaceFailed => .update_check_space_failed,
            error.WriteFilesFailed => .update_write_files_failed,
            error.WriteConfigFailed => .update_write_config_failed,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            else => null,
        },
        .uninstall => switch (err) {
            error.PackageNotFound => .uninstall_not_found,
            error.UninstallFailed => .uninstall_failed,
            error.MissingRepository, error.RepoOpenFailed => .ostree_repo_open_failed,
            error.FileMapCorrupted => .uninstall_file_map_corrupted,
            error.StagingNotCleaned => .uninstall_staging_not_cleaned,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            error.CheckoutFailed => .install_checkout_failed,
            error.Cancelled => .cancelled,
            error.MaxRetriesExceeded => .max_retries_exceeded,
            else => null,
        },
        .rollback => switch (err) {
            error.PathNotFound => .invalid_path,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            error.RollbackFailed => .ostree_rollback,
            error.NoPreviousCommit => .ostree_no_parent,
            error.CommitNotFound => .ostree_commit_not_found,
            error.StagingFailed => .ostree_staging_failed,
            error.SwapFailed => .ostree_swap_failed,
            error.CleanupFailed => .ostree_cleanup_failed,
            error.CheckSpaceFailed, error.NotEnoughSpace => .no_space_left,
            else => null,
        },
        .diff => switch (err) {
            error.PathInvalid => .invalid_path,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.CommitNotFound => .ostree_commit_not_found,
            error.DiffFailed => .diff_failed,
            error.StagingFailed => .ostree_staging_failed,
            error.CleanupFailed => .ostree_cleanup_failed,
            error.FileNotFound => .file_not_found,
            error.AllocZPrintFailed => .out_of_memory,
            error.Cancelled => .cancelled,
            else => null,
        },
        .list => switch (err) {
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.CommitNotFound => .ostree_commit_not_found,
            error.AllocFailed => .out_of_memory,
            error.ListError => .list_failed,
            error.Cancelled => .cancelled,
            error.MaxRetriesExceeded => .max_retries_exceeded,
            else => null,
        },
        .search => switch (err) {
            error.PathNotFound => .invalid_path,
            error.ReadDatabaseFailed => .db_missing_section,
            error.Cancelled => .cancelled,
            else => null,
        },
        .commit => switch (err) {
            error.PathNotFound => .invalid_path,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            error.CommitFailed => .ostree_commit,
            error.Cancelled => .cancelled,
            else => null,
        },
        .files => switch (err) {
            error.PathNotFound, error.InvalidFilePath => .invalid_path,
            error.RepoOpenFailed => .ostree_repo_open_failed,
            error.RepoTransactionFailed => .ostree_repo_transaction_failed,
            error.DatabaseNotFound, error.DatabaseReadFailed => .db_missing_section,
            error.DatabaseWriteFailed => .db_write_database_failed,
            error.PackageNotFound => .uninstall_not_found,
            error.CheckoutFailed => .install_checkout_failed,
            error.Cancelled => .cancelled,
            else => null,
        },
    };

    if (specific) |code| return code;

    return switch (err) {
        error.OutOfMemory, error.AllocZFailed => .out_of_memory,
        error.InvalidPath, error.BadPathName, error.RepoPathNotFound, error.PathNotFound => .invalid_path,
        error.FileNotFound => .file_not_found,
        error.AccessDenied => .permission_denied,
        error.AbiMismatch => .abi_mismatch,
        error.WouldBlock => .lock_would_block,
        error.ErrorTreadError => .tread_error,
        error.NotEnoughSpace => .no_space_left,
        error.Cancelled => .cancelled,
        error.MaxRetriesExceeded => .max_retries_exceeded,

        error.MissingField => .db_missing_field,
        error.MissingSection => .db_missing_section,
        error.InvalidEntry => .db_invalid_entry,
        error.ParseError => .db_parse_error,
        error.WriteDatabaseFailed => .db_write_database_failed,
        error.MalformedMeta => .db_malformed_meta,
        error.MalformedFiles => .db_malformed_files,

        error.MalformedEntry => .idx_malformed_entry,
        error.ReadFailed => .read_failed,
        error.WriteError, error.WriteFailed => .write_failed,
        error.ChecksumFailed => .file_checksum_failed,
        error.FileAlreadyExists => .file_already_exists,
        error.RepoWriteFailed => .ostree_repo_write_failed,
        error.MtreeInsertFailed => .ostree_mtree_insert_failed,

        else => .unexpected,
    };
}
