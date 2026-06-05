pub const BackendErrorCode = enum(i32) {
    ok = 0,
    checksum_mismatch = 1,
    extraction_failed = 2,
    metadata_not_found = 3,
    invalid_package = 4,
    archive_open_failed = 5,
    archive_read_failed = 6,
    archive_extract_failed = 7,
    temp_dir_failed = 8,
    alloc_failed = 9,
    cancelled = 10,
    read_failed = 11,
    invalid_entry = 12,
    abi_mismatch = 13,
    unexpected = 99,
};

pub const BackendError = error{
    ChecksumMismatch,
    ExtractionFailed,
    MetadataNotFound,
    InvalidPackage,
    ReadFailed,
    ArchiveOpenFailed,
    ArchiveReadFailed,
    ArchiveExtractFailed,
    OutOfMemory,
    TempDirFailed,
    AllocZFailed,
    Cancelled,
};

pub fn fromError(err: anyerror) BackendErrorCode {
    return switch (err) {
        BackendError.ChecksumMismatch => .checksum_mismatch,
        BackendError.ExtractionFailed => .extraction_failed,
        BackendError.MetadataNotFound => .metadata_not_found,
        BackendError.InvalidPackage => .invalid_package,
        BackendError.ArchiveOpenFailed => .archive_open_failed,
        BackendError.ArchiveReadFailed => .archive_read_failed,
        BackendError.ArchiveExtractFailed => .archive_extract_failed,
        BackendError.TempDirFailed => .temp_dir_failed,
        BackendError.AllocZFailed, BackendError.OutOfMemory => .alloc_failed,
        BackendError.Cancelled => .cancelled,
        BackendError.ReadFailed => .read_failed,
        error.InvalidEntry => .invalid_entry,
        error.AbiMismatch => .abi_mismatch,
        else => .unexpected,
    };
}
