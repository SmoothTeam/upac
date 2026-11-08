// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::decoder::DecodeError;
use super::error::ErrorDomain;
use super::package::{DecodedPackageMeta, PackageChecksum};

pub trait CommandState: Copy {
    const DOMAIN: ErrorDomain;
    const VALIDATION: Self;

    fn as_u32(self) -> u32;

    fn from_stage_index(index: usize) -> Self;
}

pub trait DecodeMeta {
    fn decode(&self, checksum: PackageChecksum) -> Result<DecodedPackageMeta, DecodeError>;
}
