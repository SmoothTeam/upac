// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use redb::ReadableDatabase;

use uuid::Uuid;

use upac_types::decoder::DeclarativeTrigger;

use super::error::DatabaseError;
use super::{
    MemoryDatabase, PACKAGES_TRIGGERS_TABLE, ReadTransactionExt, ReadableSource, record_decode, record_encode,
};

pub trait TriggerStore {
    fn get_declarative_triggers(&self, uuid: Uuid) -> Result<Option<DeclarativeTrigger>, DatabaseError>;
}

pub trait TriggerStoreMut: TriggerStore {
    fn set_declarative_triggers(&mut self, uuid: Uuid, trigger: &DeclarativeTrigger) -> Result<(), DatabaseError>;
    fn remove_declarative_triggers(&mut self, uuid: Uuid) -> Result<(), DatabaseError>;
}

impl<T: ReadableSource> TriggerStore for T {
    fn get_declarative_triggers(&self, uuid: Uuid) -> Result<Option<DeclarativeTrigger>, DatabaseError> {
        let transaction = self.source().begin_read()?;
        let Some(triggers) = transaction.open_table_or_none(PACKAGES_TRIGGERS_TABLE)? else {
            return Ok(None);
        };

        triggers
            .get(uuid)?
            .map(|guard| record_decode(guard.value()))
            .transpose()
    }
}

impl TriggerStoreMut for MemoryDatabase {
    fn set_declarative_triggers(&mut self, uuid: Uuid, trigger: &DeclarativeTrigger) -> Result<(), DatabaseError> {
        let transaction = self.database.begin_write()?;

        transaction
            .open_table(PACKAGES_TRIGGERS_TABLE)?
            .insert(uuid, record_encode(trigger)?.as_slice())?;

        transaction.commit()?;
        Ok(())
    }

    fn remove_declarative_triggers(&mut self, uuid: Uuid) -> Result<(), DatabaseError> {
        let transaction = self.database.begin_write()?;

        transaction.open_table(PACKAGES_TRIGGERS_TABLE)?.remove(uuid)?;

        transaction.commit()?;
        Ok(())
    }
}
