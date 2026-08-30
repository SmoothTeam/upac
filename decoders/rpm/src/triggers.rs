// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::DecoderTrigger;

use crate::header::Header;
use crate::rpm::{POSTIN_NAME, POSTIN_TAG, POSTUN_NAME, POSTUN_TAG, PREIN_NAME, PREIN_TAG, PREUN_NAME, PREUN_TAG};

pub fn scan(header: &Header) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for trigger in DecoderTrigger::ALL {
        let (tag, name) = native(trigger);
        let already_added = names.iter().any(|existing| existing == name);

        if header.contains(tag) && !already_added {
            names.push(name.to_owned());
        }
    }

    names
}

fn native(trigger: DecoderTrigger) -> (u32, &'static str) {
    match trigger {
        DecoderTrigger::PreInstall | DecoderTrigger::PreUpgrade => (PREIN_TAG, PREIN_NAME),
        DecoderTrigger::PostInstall | DecoderTrigger::PostUpgrade => (POSTIN_TAG, POSTIN_NAME),
        DecoderTrigger::PreRemove => (PREUN_TAG, PREUN_NAME),
        DecoderTrigger::PostRemove => (POSTUN_TAG, POSTUN_NAME),
    }
}
