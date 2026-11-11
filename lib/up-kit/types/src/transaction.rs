// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use upac_macro::CEnum;

use crate::error::ErrorKind;
use crate::layout::transaction::{BOOTSTRAP, FILES, INSTALL, UNINSTALL, UPDATE};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, CEnum)]
pub enum TransactionKind {
    Bootstrap = 0,
    Install = 1,
    Update = 2,
    Uninstall = 3,
    Files = 4,
}

impl TransactionKind {
    pub const ALL: [TransactionKind; 5] = [
        TransactionKind::Bootstrap,
        TransactionKind::Install,
        TransactionKind::Update,
        TransactionKind::Uninstall,
        TransactionKind::Files,
    ];
}

impl Display for TransactionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(match self {
            TransactionKind::Bootstrap => BOOTSTRAP,
            TransactionKind::Install => INSTALL,
            TransactionKind::Update => UPDATE,
            TransactionKind::Uninstall => UNINSTALL,
            TransactionKind::Files => FILES,
        })
    }
}

impl FromStr for TransactionKind {
    type Err = ErrorKind;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        TransactionKind::ALL
            .into_iter()
            .find(|kind| kind.to_string() == name)
            .ok_or(ErrorKind::InvalidEntry)
    }
}
