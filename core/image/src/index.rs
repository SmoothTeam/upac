// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use upac_database::{PackageEntryId, PackageId};

#[derive(Debug, Default)]
pub(crate) struct ImageIndex {
    places: BTreeMap<PathBuf, PackageEntryId>,
    nodes: HashMap<PackageEntryId, PathBuf>,
    packages: HashMap<PackageId, HashSet<PackageEntryId>>,
}
