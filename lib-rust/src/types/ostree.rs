use std::path::{Component, Path};

use gio::{Cancellable, File as GioFile, IOErrorEnum};
use glib::prelude::Cast;
use ostree::{MutableTree, Repo, RepoFile, RepoTransactionStats};

use crate::types::errors::CommonError;

macro_rules! map_glib_error {
    ($result:expr, $fallback:expr, { $($io_variant:pat => $mapped:expr),* $(,)? }) => {
        match $result {
            Ok(value) => Ok(value),
            Err(error) => match error.kind::<IOErrorEnum>() {
                $(Some($io_variant) => Err($mapped),)*
                _ => Err($fallback),
            },
        }
    };
}

impl From<GioFile> for CommonError {
    fn from(_: GioFile) -> Self {
        CommonError::MtreeWriteFailed
    }
}

pub struct RepoHandle {
    repo: Repo,
}

impl RepoHandle {
    pub fn open(path: &str, cancellable: Option<&Cancellable>) -> Result<Self, CommonError> {
        let repo = Repo::new(&GioFile::for_path(path));

        map_glib_error!(repo.open(cancellable), CommonError::RepoOpenFailed, {})?;

        Ok(Self { repo })
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn begin_transaction(&self, cancellable: Option<&Cancellable>) -> Result<bool, CommonError> {
        map_glib_error!(
            self.repo.prepare_transaction(cancellable),
            CommonError::RepoTransactionFailed,
            {}
        )
    }

    pub fn commit_transaction(&self, cancellable: Option<&Cancellable>) -> Result<RepoTransactionStats, CommonError> {
        map_glib_error!(
            self.repo.commit_transaction(cancellable),
            CommonError::RepoTransactionFailed,
            {}
        )
    }

    pub fn abort_transaction(&self, cancellable: Option<&Cancellable>) -> Result<(), CommonError> {
        map_glib_error!(
            self.repo.abort_transaction(cancellable),
            CommonError::RepoTransactionFailed,
            {}
        )
    }
}

pub struct CommitBuilder<'repo> {
    repo: &'repo RepoHandle,
    mtree: MutableTree,
    subject: Option<String>,
    body: Option<String>,
    branch: Option<String>,
}

impl<'repo> CommitBuilder<'repo> {
    pub fn new(repo: &'repo RepoHandle, mtree: MutableTree) -> Self {
        Self {
            repo,
            mtree,
            subject: None,
            body: None,
            branch: None,
        }
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn build(self, cancellable: Option<&Cancellable>) -> Result<Commit<'repo>, CommonError> {
        let root_file = map_glib_error!(
            self.repo.repo().write_mtree(&self.mtree, cancellable),
            CommonError::MtreeWriteFailed,
            {}
        )?;

        let root: RepoFile = root_file.downcast()?;

        Ok(Commit {
            repo: self.repo,
            root,
            subject: self.subject,
            body: self.body,
            branch: self.branch,
        })
    }
}

pub struct Commit<'repo> {
    repo: &'repo RepoHandle,
    root: RepoFile,
    subject: Option<String>,
    body: Option<String>,
    branch: Option<String>,
}

impl Commit<'_> {
    pub fn commit(self, parent: Option<&str>, cancellable: Option<&Cancellable>) -> Result<String, CommonError> {
        let checksum = map_glib_error!(
            self.repo.repo().write_commit(
                parent,
                self.subject.as_deref(),
                self.body.as_deref(),
                None,
                &self.root,
                cancellable
            ),
            CommonError::CommitWriteFailed,
            {}
        )?;

        if let Some(branch) = &self.branch {
            self.repo
                .repo()
                .transaction_set_ref(None, branch, Some(checksum.as_str()));
        }

        Ok(checksum.to_string())
    }
}

pub trait MutableTreeExt {
    fn is_empty(&self) -> bool;
    fn insert_dir(&self, path: &str, metadata_checksum: &str) -> Result<(), CommonError>;
    fn insert_file(&self, path: &str, content_checksum: &str) -> Result<(), CommonError>;
    fn remove_path(&self, path: &str) -> Result<(), CommonError>;
}

impl MutableTreeExt for MutableTree {
    fn is_empty(&self) -> bool {
        self.copy_files().is_empty() && self.copy_subdirs().is_empty()
    }

    fn insert_dir(&self, path: &str, metadata_checksum: &str) -> Result<(), CommonError> {
        let path_segments: Vec<&str> = Path::new(path)
            .components()
            .map(|path_component| match path_component {
                Component::Normal(path_segment) => path_segment.to_str().ok_or(CommonError::MtreeInsertFailed),
                _ => Err(CommonError::MtreeInsertFailed),
            })
            .collect::<Result<Vec<&str>, CommonError>>()?;
        let mut current_node = self.clone();

        for path_segment in &path_segments {
            current_node = map_glib_error!(current_node.ensure_dir(path_segment), CommonError::MtreeInsertFailed, {
            })?;
        }

        current_node.set_metadata_checksum(metadata_checksum);

        Ok(())
    }

    fn insert_file(&self, path: &str, content_checksum: &str) -> Result<(), CommonError> {
        let path_segments: Vec<&str> = Path::new(path)
            .components()
            .map(|path_component| match path_component {
                Component::Normal(path_segment) => path_segment.to_str().ok_or(CommonError::MtreeInsertFailed),
                _ => Err(CommonError::MtreeInsertFailed),
            })
            .collect::<Result<Vec<&str>, CommonError>>()?;

        let (&file_name, parent_segments) = path_segments.split_last().ok_or(CommonError::MtreeInsertFailed)?;

        let parent_node = map_glib_error!(self.walk(parent_segments, 0), CommonError::MtreeInsertFailed, {})?;

        map_glib_error!(
            parent_node.replace_file(file_name, content_checksum),
            CommonError::MtreeInsertFailed,
            {}
        )
    }

    fn remove_path(&self, path: &str) -> Result<(), CommonError> {
        let path_segments: Vec<&str> = Path::new(path)
            .components()
            .map(|path_component| match path_component {
                Component::Normal(path_segment) => path_segment.to_str().ok_or(CommonError::MtreeInsertFailed),
                _ => Err(CommonError::MtreeInsertFailed),
            })
            .collect::<Result<Vec<&str>, CommonError>>()?;

        let (&entry_name, parent_segments) = path_segments.split_last().ok_or(CommonError::MtreeInsertFailed)?;

        let mut ancestor_nodes = vec![self.clone()];
        let mut current_node = self.clone();

        for &path_segment in parent_segments {
            current_node = map_glib_error!(current_node.walk(&[path_segment], 0), CommonError::MtreeInsertFailed, {})?;
            ancestor_nodes.push(current_node.clone());
        }

        let parent_node = &ancestor_nodes[parent_segments.len()];

        map_glib_error!(parent_node.remove(entry_name, true), CommonError::MtreeInsertFailed, {})?;

        for ancestor_index in (1..ancestor_nodes.len()).rev() {
            let ancestor_node = &ancestor_nodes[ancestor_index];

            if !ancestor_node.is_empty() {
                break;
            }

            let ancestor_parent_node = &ancestor_nodes[ancestor_index - 1];
            let ancestor_name = parent_segments[ancestor_index - 1];

            map_glib_error!(ancestor_parent_node.remove(ancestor_name, true), CommonError::MtreeInsertFailed, {})?;
        }

        Ok(())
    }
}
