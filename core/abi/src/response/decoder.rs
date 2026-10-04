// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_macro::{CFree, CNew, CValidate};

use crate::package::{CPackageDependency, CPackageMeta, CPackageTrigger};
use crate::plugin::FreeDecodeResponseFn;
use crate::types::CVec;

#[repr(C)]
#[derive(CFree, CNew, CValidate)]
pub struct CDecodeResponse {
    pub struct_size: usize,

    pub meta: CPackageMeta,

    pub dependencies: CVec<CPackageDependency>,
    pub triggers: CVec<CPackageTrigger>,

    pub free: FreeDecodeResponseFn,
}

impl Drop for CDecodeResponse {
    fn drop(&mut self) {
        unsafe { (self.free)(self) };
    }
}
