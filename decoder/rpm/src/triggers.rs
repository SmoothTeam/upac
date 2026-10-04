// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::{PackageTrigger, TriggerPosition};

use super::header::Header;
use super::rpm::{POSTIN_NAME, POSTIN_TAG, POSTUN_NAME, POSTUN_TAG, PREIN_NAME, PREIN_TAG, PREUN_NAME, PREUN_TAG};

pub fn scan(header: &Header) -> Vec<PackageTrigger> {
    TriggerPosition::ALL
        .into_iter()
        .filter_map(|position| {
            let (tag, name) = native(position);

            header.contains(tag).then(|| PackageTrigger {
                position,
                name: name.to_owned(),
            })
        })
        .collect()
}

fn native(position: TriggerPosition) -> (u32, &'static str) {
    match position {
        TriggerPosition::PreInstall | TriggerPosition::PreUpgrade => (PREIN_TAG, PREIN_NAME),
        TriggerPosition::PostInstall | TriggerPosition::PostUpgrade => (POSTIN_TAG, POSTIN_NAME),
        TriggerPosition::PreRemove => (PREUN_TAG, PREUN_NAME),
        TriggerPosition::PostRemove => (POSTUN_TAG, POSTUN_NAME),
    }
}
