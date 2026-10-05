// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cmp::Reverse;

use upac_types::CancelToken;
use upac_types::error::ErrorKind;

use upac_deploy::Sysroot;
use upac_deploy::deployment::{Deployment, PrefixDeploy};
use upac_deploy::error::{PrefixMetaError, PrefixReadError};

pub mod attach;
pub mod commit;
pub mod detach;
pub mod gc;
pub mod installer;
pub mod mime;
pub mod pin;
pub mod rollback;
pub mod stages;
pub mod uninstaller;
pub mod update;

pub(crate) struct TmpPath(pub String);

pub(crate) fn prune_prefixes(
    sysroot: &Sysroot, retention_depth: usize, cancel: &CancelToken, progress: &dyn Fn(Option<&str>, u64, u64),
) -> Result<Vec<PrefixDeploy>, ErrorKind> {
    let running_digest = sysroot.running_prefix()?.digest().clone();
    let next_digest = match sysroot.next_prefix() {
        Ok(next_prefix) => Some(next_prefix.digest().clone()),
        Err(PrefixReadError::Meta(PrefixMetaError::NotFound)) => None,
        Err(error) => return Err(error.into()),
    };

    let mut prefixes = sysroot.prefixes()?;
    prefixes.sort_by_key(|prefix| Reverse(prefix.timestamp()));

    let (retained, removed): (Vec<_>, Vec<_>) = prefixes.into_iter().enumerate().partition(|(age_rank, prefix)| {
        *age_rank < retention_depth
            || prefix.pinned()
            || prefix.digest() == &running_digest
            || Some(prefix.digest()) == next_digest.as_ref()
    });

    let total = removed.len() as u64;
    for (position, (_, prefix)) in removed.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(ErrorKind::Cancelled);
        }

        sysroot.remove_prefix(prefix.digest())?;
        progress(Some(prefix.digest().to_hex().as_str()), position as u64 + 1, total);
    }

    Ok(retained.into_iter().map(|(_, prefix)| prefix).collect())
}
