// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::BTreeMap;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

use composefs::generic_tree::Stat;

pub trait ToStat {
    fn to_stat(&self) -> Stat;
}

impl ToStat for Metadata {
    fn to_stat(&self) -> Stat {
        Stat {
            st_mode: self.mode(),
            st_uid: self.uid(),
            st_gid: self.gid(),
            st_mtim_sec: self.mtime(),
            st_mtim_nsec: self.mtime_nsec() as u32,
            xattrs: BTreeMap::new(),
        }
    }
}
