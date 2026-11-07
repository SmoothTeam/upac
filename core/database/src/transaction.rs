// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::from_utf8;

use composefs::generic_tree::{LeafContent, Stat};
use composefs::tree::RegularFile;

use upac_composefs::tree::Tree;
use upac_composefs::tree::builder::TreeBuilder;

use upac_types::transaction::Transaction;

use super::error::DatabaseError;
use super::layout::store::{DIR_MODE, FILE_MODE, ROOT_DIR};
use super::layout::transaction::{DIR, MESSAGE_XATTR, SEPARATOR, SUBJECT_XATTR};

pub struct TransactionStore<TreeAccess> {
    tree: TreeAccess,
}

impl<'tree> TransactionStore<&'tree Tree> {
    pub fn read(tree: &'tree Tree) -> Self {
        Self { tree }
    }

    pub fn list(&self) -> Result<Vec<Transaction>, DatabaseError> {
        let directory = PathBuf::from(ROOT_DIR).join(DIR);

        if !self.tree.check(&directory) {
            return Ok(Vec::new());
        }

        let mut transactions = Vec::new();
        for record_name in self.tree.list(&directory)? {
            let stat = self.tree.get_metadata(directory.join(record_name))?;

            transactions.push(self.transaction_parse(record_name, stat)?);
        }
        transactions.sort_by_key(|transaction| transaction.timestamp);

        Ok(transactions)
    }
}

impl<'tree> TransactionStore<&'tree mut TreeBuilder> {
    pub fn write(builder: &'tree mut TreeBuilder) -> Self {
        Self { tree: builder }
    }

    pub fn init(&mut self) -> Result<(), DatabaseError> {
        let stat = Stat {
            st_mode: DIR_MODE,
            st_uid: 0,
            st_gid: 0,
            st_mtim_sec: 0,
            st_mtim_nsec: 0,
            xattrs: BTreeMap::new(),
        };

        self.tree.insert_dir(PathBuf::from(ROOT_DIR).join(DIR), stat)?;

        Ok(())
    }

    pub fn append(&mut self, transaction: &Transaction) -> Result<(), DatabaseError> {
        let mut xattrs = BTreeMap::from([(OsStr::new(SUBJECT_XATTR).into(), transaction.subject.as_bytes().into())]);

        if let Some(message) = &transaction.message {
            xattrs.insert(OsStr::new(MESSAGE_XATTR).into(), message.as_bytes().into());
        }

        let stat = Stat {
            st_mode: FILE_MODE,
            st_uid: 0,
            st_gid: 0,
            st_mtim_sec: 0,
            st_mtim_nsec: 0,
            xattrs,
        };

        let path = PathBuf::from(ROOT_DIR)
            .join(DIR)
            .join(format!("{}{}{}", transaction.timestamp, SEPARATOR, transaction.kind));
        let leaf = LeafContent::Regular(RegularFile::Inline(Box::default()));

        self.tree.insert(path, leaf, stat)?;

        Ok(())
    }
}

impl<TreeAccess> TransactionStore<TreeAccess> {
    fn transaction_parse(&self, record_name: &OsStr, stat: &Stat) -> Result<Transaction, DatabaseError> {
        let (timestamp, kind) = record_name
            .to_str()
            .and_then(|record_name| record_name.split_once(SEPARATOR))
            .ok_or(DatabaseError::InvalidTransaction)?;

        Ok(Transaction {
            timestamp: timestamp.parse().map_err(|_| DatabaseError::InvalidTransaction)?,
            kind: kind.parse().map_err(|_| DatabaseError::InvalidTransaction)?,
            subject: self
                .xattr_text(stat, SUBJECT_XATTR)?
                .ok_or(DatabaseError::InvalidTransaction)?,
            message: self.xattr_text(stat, MESSAGE_XATTR)?,
        })
    }

    fn xattr_text(&self, stat: &Stat, name: &str) -> Result<Option<String>, DatabaseError> {
        stat.xattrs
            .get(OsStr::new(name))
            .map(|value| {
                from_utf8(value)
                    .map(str::to_owned)
                    .map_err(|_| DatabaseError::InvalidTransaction)
            })
            .transpose()
    }
}
