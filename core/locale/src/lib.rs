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

pub fn parse<Cli: Parser, Localization: CliLocale>() -> Cli {
    try_parse_from::<Cli, Localization, _, _>(args_os()).unwrap_or_else(|error| exit::<Localization>(error))
}

pub fn try_parse_from<Cli, Localization, Arguments, Argument>(arguments: Arguments) -> Result<Cli, ClapError>
where
    Cli: Parser,
    Localization: CliLocale,
    Arguments: IntoIterator<Item = Argument>,
    Argument: Into<OsString> + Clone,
{
    let mut command = localize::<Localization>(Cli::command());

    let mut matches = command.try_get_matches_from_mut(arguments)?;

    Cli::from_arg_matches_mut(&mut matches).map_err(|error| error.format(&mut command))
}
