use upac_abi::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoError {
    Open,
    Transaction,
    MtreeWrite,
    MtreeInsert,
    CommitWrite,
}

impl From<RepoError> for ErrorKind {
    fn from(error: RepoError) -> Self {
        match error {
            RepoError::Open => ErrorKind::Unexpected,
            RepoError::Transaction => ErrorKind::WriteFailed,
            RepoError::MtreeWrite => ErrorKind::WriteFailed,
            RepoError::MtreeInsert => ErrorKind::WriteFailed,
            RepoError::CommitWrite => ErrorKind::WriteFailed,
        }
    }
}
