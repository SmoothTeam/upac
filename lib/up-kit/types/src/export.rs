// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::panic::{AssertUnwindSafe, catch_unwind};

use upac_abi::error::CError;

use crate::error::{Error, ErrorKind};
use crate::request::Output;
use crate::state::CommandState;

/// # Safety
/// `err_out`, if non-null, must point to writable `CError` storage.
pub unsafe fn export_command<'request, CRequest, Request, State>(
    request: &'request CRequest, err_out: *mut CError,
    run: impl FnOnce(Request) -> Result<(), (State, ErrorKind, Option<String>)>,
) -> i32
where
    Request: TryFrom<&'request CRequest, Error = ErrorKind>,
    State: CommandState,
{
    let request = match Request::try_from(request) {
        Ok(request) => request,
        Err(kind) => return unsafe { write_error(err_out, Error::new(State::VALIDATION, kind)) },
    };

    match catch_unwind(AssertUnwindSafe(|| run(request))) {
        Ok(Ok(())) => 0,
        Ok(Err((state, kind, subject))) => unsafe {
            write_error(err_out, Error::new(state, kind).with_subject(subject))
        },
        Err(_) => unsafe { write_error(err_out, Error::new(State::VALIDATION, ErrorKind::Unexpected)) },
    }
}

/// # Safety
/// `request`, if non-null, must point to a valid, writable `CRequest` whose output fields are empty, for the
/// duration of the call. `err_out`, if non-null, must point to writable `CError` storage.
pub unsafe fn export_command_with_output<'request, CRequest, Request, State, Value>(
    request: *mut CRequest, err_out: *mut CError,
    run: impl FnOnce(Request) -> Result<Value, (State, ErrorKind, Option<String>)>,
) -> i32
where
    CRequest: Output<'request, Value = Value> + 'request,
    Request: TryFrom<&'request CRequest, Error = ErrorKind>,
    State: CommandState,
{
    let Some(borrowed_request) = (unsafe { request.as_ref() }) else {
        return unsafe { write_error(err_out, Error::new(State::VALIDATION, ErrorKind::InvalidEntry)) };
    };

    let converted_request = match Request::try_from(borrowed_request) {
        Ok(converted_request) => converted_request,
        Err(kind) => return unsafe { write_error(err_out, Error::new(State::VALIDATION, kind)) },
    };

    match catch_unwind(AssertUnwindSafe(|| run(converted_request))) {
        Ok(Ok(value)) => {
            unsafe { (*request).write_output(value) };

            0
        }
        Ok(Err((state, kind, subject))) => unsafe {
            write_error(err_out, Error::new(state, kind).with_subject(subject))
        },
        Err(_) => unsafe { write_error(err_out, Error::new(State::VALIDATION, ErrorKind::Unexpected)) },
    }
}

unsafe fn write_error(err_out: *mut CError, error: Error) -> i32 {
    if !err_out.is_null() {
        unsafe { err_out.write(error.into()) };
    }

    -1
}
