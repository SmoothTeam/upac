// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_abi::error::CError;
use upac_abi::request::partition::{CSetupPartitionAddRequest, CSetupPartitionTableRequest};
use upac_abi::response::partition::CSetupPartitionAddResponse;

use upac_types::error::{Error, try_convert_abi, write_abi_error};
use upac_types::request::partition::{SetupPartitionAddRequest, SetupPartitionTableRequest};
use upac_types::state::setup::{PartitionAddStateId, PartitionTableStateId};

use crate::commands::partition::{add, table};

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `err_out`, if non-null, must point to writable `CError` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_table(request_c: CSetupPartitionTableRequest, err_out: *mut CError) -> i32 {
    let partition_table_request = try_convert_abi!(
        SetupPartitionTableRequest::try_from(&request_c),
        err_out,
        PartitionTableStateId
    );

    match Error::catch(|| table::run(partition_table_request)) {
        Ok(()) => 0,

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}

/// # Safety
/// Any borrowed byte-slice fields inside `request_c` must remain valid for the duration of the
/// call. `response_out` and `err_out`, if non-null, must each point to writable storage of the
/// matching type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition_add(
    request_c: CSetupPartitionAddRequest, response_out: *mut CSetupPartitionAddResponse, err_out: *mut CError,
) -> i32 {
    let partition_add_request = try_convert_abi!(
        SetupPartitionAddRequest::try_from(&request_c),
        err_out,
        PartitionAddStateId
    );

    match Error::catch(|| add::run(partition_add_request)) {
        Ok(response) => {
            if !response_out.is_null() {
                unsafe { *response_out = response.into() };
            }
            0
        }

        Err(error) => unsafe { write_abi_error(err_out, error) },
    }
}
