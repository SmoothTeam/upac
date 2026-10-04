// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use redb::ReadableDatabase;

use uuid::Uuid;

use upac_types::decoder::PackageTriggers;

use super::error::DatabaseError;
use super::{MemoryDatabase, PACKAGES_TRIGGERS_TABLE, ReadTransactionExt, record_decode, record_encode};

pub trait TriggerStore {
    fn get_package_triggers(&self, uuid: Uuid) -> Result<Option<PackageTriggers>, DatabaseError>;
}

pub trait TriggerStoreMut: TriggerStore {
    fn set_package_triggers(&mut self, uuid: Uuid, triggers: &PackageTriggers) -> Result<(), DatabaseError>;
    fn remove_package_triggers(&mut self, uuid: Uuid) -> Result<(), DatabaseError>;
}

impl TriggerStore for MemoryDatabase {
    fn get_package_triggers(&self, uuid: Uuid) -> Result<Option<PackageTriggers>, DatabaseError> {
        let transaction = self.database.begin_read()?;
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
    fn set_package_triggers(&mut self, uuid: Uuid, triggers: &PackageTriggers) -> Result<(), DatabaseError> {
        let transaction = self.database.begin_write()?;

        transaction
            .open_table(PACKAGES_TRIGGERS_TABLE)?
            .insert(uuid, record_encode(triggers)?.as_slice())?;

        transaction.commit()?;
        Ok(())
    }

    fn remove_package_triggers(&mut self, uuid: Uuid) -> Result<(), DatabaseError> {
        let transaction = self.database.begin_write()?;

        transaction.open_table(PACKAGES_TRIGGERS_TABLE)?.remove(uuid)?;

        transaction.commit()?;
        Ok(())
    }
}
