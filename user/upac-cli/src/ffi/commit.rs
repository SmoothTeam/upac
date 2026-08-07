// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use super::ctypes::CSlice;

// ── CCommitEntry ──────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CCommitEntry {
    struct_size: usize,
    pub checksum: CSlice,
    pub subject: CSlice,
}
