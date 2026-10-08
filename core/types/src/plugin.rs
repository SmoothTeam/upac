// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

const PLUGIN_SUCCESS_STATUS: i32 = 0;

pub fn plugin_status<StatusError: Into<i32>>(result: Result<(), StatusError>) -> i32 {
    match result {
        Ok(()) => PLUGIN_SUCCESS_STATUS,
        Err(error) => error.into(),
    }
}

pub fn plugin_result<StatusError: TryFrom<i32>>(status: i32) -> Result<(), Option<StatusError>> {
    if status == PLUGIN_SUCCESS_STATUS {
        return Ok(());
    }

    Err(StatusError::try_from(status).ok())
}
