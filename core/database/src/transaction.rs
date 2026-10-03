// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use redb::ReadableDatabase;

use upac_types::transaction::Transaction;

use super::error::DatabaseError;
use super::{MemoryDatabase, ReadTransactionExt, ReadableSource, TRANSACTION_TABLE, record_decode, record_encode};

pub trait TransactionStore {
    fn get_transaction(&self) -> Result<Option<Transaction>, DatabaseError>;
}

pub trait TransactionStoreMut: TransactionStore {
    fn set_transaction(&mut self, transaction: &Transaction) -> Result<(), DatabaseError>;
}

impl<T: ReadableSource> TransactionStore for T {
    fn get_transaction(&self) -> Result<Option<Transaction>, DatabaseError> {
        let read_transaction = self.source().begin_read()?;
        let Some(table) = read_transaction.open_table_or_none(TRANSACTION_TABLE)? else {
            return Ok(None);
        };

        table.get(())?.map(|guard| record_decode(guard.value())).transpose()
    }
}

impl TransactionStoreMut for MemoryDatabase {
    fn set_transaction(&mut self, transaction: &Transaction) -> Result<(), DatabaseError> {
        let write_transaction = self.database.begin_write()?;

        write_transaction
            .open_table(TRANSACTION_TABLE)?
            .insert((), record_encode(transaction)?.as_slice())?;

        write_transaction.commit()?;
        Ok(())
    }
}
