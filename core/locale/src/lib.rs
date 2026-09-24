// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::args_os;
use std::ffi::OsString;

use clap::Error as ClapError;
use clap::Parser;

use i18n_embed::fluent::FluentLanguageLoader;

use self::command::localize;
use self::error::exit;

pub mod command;
pub mod error;

pub trait CliLocale: 'static {
    fn loader() -> &'static FluentLanguageLoader;
}

pub fn parse<C: Parser, L: CliLocale>() -> C {
    try_parse_from::<C, L, _, _>(args_os()).unwrap_or_else(|error| exit::<L>(error))
}

pub fn try_parse_from<C, L, I, T>(arguments: I) -> Result<C, ClapError>
where
    C: Parser,
    L: CliLocale,
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut command = localize::<L>(C::command());

    let mut matches = command.try_get_matches_from_mut(arguments)?;

    C::from_arg_matches_mut(&mut matches).map_err(|error| error.format(&mut command))
}
