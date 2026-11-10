// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::any::{Any, TypeId, type_name};
use std::collections::{HashMap, HashSet};

use super::error::PipelineError;

#[derive(Default)]
pub struct Context {
    slots: HashMap<TypeId, Box<dyn Any>>,
}

impl Context {
    pub fn put<Slot: Any>(&mut self, value: Slot) {
        let previous = self.slots.insert(TypeId::of::<Slot>(), Box::new(value));

        debug_assert!(
            previous.is_none(),
            "context slot `{}` was already filled; use `replace` to overwrite it on purpose",
            type_name::<Slot>()
        );
    }

    pub fn replace<Slot: Any>(&mut self, value: Slot) -> Option<Slot> {
        self.slots
            .insert(TypeId::of::<Slot>(), Box::new(value))
            .and_then(|slot| slot.downcast::<Slot>().ok())
            .map(|boxed| *boxed)
    }

    pub fn push<Slot: Any>(&mut self, value: Slot) {
        let slot = self
            .slots
            .entry(TypeId::of::<Vec<Slot>>())
            .or_insert_with(|| Box::new(Vec::<Slot>::new()));

        if let Some(list) = slot.downcast_mut::<Vec<Slot>>() {
            list.push(value);
        }
    }

    pub fn get<Slot: Any>(&self) -> Result<&Slot, PipelineError> {
        self.slots
            .get(&TypeId::of::<Slot>())
            .and_then(|slot| slot.downcast_ref::<Slot>())
            .ok_or(PipelineError::MissingResult)
    }

    pub fn take<Slot: Any>(&mut self) -> Result<Slot, PipelineError> {
        self.slots
            .remove(&TypeId::of::<Slot>())
            .and_then(|slot| slot.downcast::<Slot>().ok())
            .map(|boxed| *boxed)
            .ok_or(PipelineError::MissingResult)
    }

    pub(crate) fn type_ids(&self) -> HashSet<TypeId> {
        self.slots.keys().copied().collect()
    }

    pub(crate) fn put_boxed(&mut self, type_id: TypeId, value: Box<dyn Any>) {
        self.slots.insert(type_id, value);
    }

    pub(crate) fn discard(&mut self, type_id: TypeId) {
        self.slots.remove(&type_id);
    }
}
