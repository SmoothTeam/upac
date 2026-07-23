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

pub fn fromError(err: anyerror) BackendErrorCode {
    return switch (err) {
        error.ChecksumMismatch => .checksum_mismatch,
        error.ExtractionFailed => .extraction_failed,
        error.MetadataNotFound => .metadata_not_found,
        error.InvalidPackage => .invalid_package,
        error.ArchiveOpenFailed => .archive_open_failed,
        error.ArchiveReadFailed => .archive_read_failed,
        error.ArchiveExtractFailed => .archive_extract_failed,
        error.TempDirFailed => .temp_dir_failed,
        error.AllocZFailed, error.OutOfMemory => .alloc_failed,
        error.Cancelled => .cancelled,
        error.ReadFailed => .read_failed,
        error.InvalidEntry => .invalid_entry,
        error.AbiMismatch => .abi_mismatch,
        else => .unexpected,
    };
}
