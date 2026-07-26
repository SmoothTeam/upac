use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoError {
    OpenFailed,
    TransactionFailed,
    MtreeWriteFailed,
    MtreeInsertFailed,
    CommitWriteFailed,
}

impl From<RepoError> for ErrorKind {
    fn from(error: RepoError) -> Self {
        match error {
            RepoError::OpenFailed => ErrorKind::Unexpected,
            RepoError::TransactionFailed => ErrorKind::WriteFailed,
            RepoError::MtreeWriteFailed => ErrorKind::WriteFailed,
            RepoError::MtreeInsertFailed => ErrorKind::WriteFailed,
            RepoError::CommitWriteFailed => ErrorKind::WriteFailed,
        }
    }
}
