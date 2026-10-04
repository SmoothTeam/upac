// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::sync::atomic::AtomicU8;

use upac_macro::{CFree, CNew, CValidate};

use crate::types::CSlice;

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDecodeRequest {
    pub struct_size: usize,

    pub package_path: CSlice,
    pub output_dir: CSlice,

    pub checksum: [u8; 32],

    pub cancel_token: *const AtomicU8,
}
