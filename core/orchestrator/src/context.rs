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
    pub fn put<T: Any>(&mut self, value: T) {
        let previous = self.slots.insert(TypeId::of::<T>(), Box::new(value));

        debug_assert!(
            previous.is_none(),
            "context slot `{}` was already filled; use `replace` to overwrite it on purpose",
            type_name::<T>()
        );
    }

    pub fn replace<T: Any>(&mut self, value: T) -> Option<T> {
        self.slots
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|slot| slot.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn push<T: Any>(&mut self, value: T) {
        let slot = self
            .slots
            .entry(TypeId::of::<Vec<T>>())
            .or_insert_with(|| Box::new(Vec::<T>::new()));

        if let Some(list) = slot.downcast_mut::<Vec<T>>() {
            list.push(value);
        }
    }

    pub fn get<T: Any>(&self) -> Result<&T, PipelineError> {
        self.slots
            .get(&TypeId::of::<T>())
            .and_then(|slot| slot.downcast_ref::<T>())
            .ok_or(PipelineError::MissingResult)
    }

    pub fn take<T: Any>(&mut self) -> Result<T, PipelineError> {
        self.slots
            .remove(&TypeId::of::<T>())
            .and_then(|slot| slot.downcast::<T>().ok())
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
