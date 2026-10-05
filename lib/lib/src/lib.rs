// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::hook::CProgressEvent;

use upac_types::progress::ProgressEvent;
use upac_types::request::RequestBase;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod export;
mod mutated;
mod search;
mod unmutated;

pub(crate) fn report_progress(base: &RequestBase<'_>, event: &ProgressEvent<'_>) {
    let Some(on_hook) = base.on_hook else {
        return;
    };

    let c_event = CProgressEvent::from(event.clone());
    unsafe { on_hook(&c_event, base.hook_ctx) };
    unsafe { c_event.free() };
}
