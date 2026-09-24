// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;
use std::marker::PhantomData;

use clap::builder::StyledStr;
use clap::builder::styling::Styles;
use clap::error::Error as ClapError;
use clap::error::{ContextKind, ContextValue, ErrorFormatter, ErrorKind};

use super::CliLocale;

pub struct LocalizedErrorFormatter<L>(PhantomData<L>);

impl<L: CliLocale> ErrorFormatter for LocalizedErrorFormatter<L> {
    fn format_error(error: &ClapError<Self>) -> StyledStr {
        let loader = L::loader();
        let styles = Styles::default();
        let error_style = styles.get_error();
        let usage_style = styles.get_usage();

        let message = match error.kind() {
            ErrorKind::MissingRequiredArgument => loader.get_args(
                "clap-error-missing-argument",
                HashMap::from([("arguments", context(error, ContextKind::InvalidArg))]),
            ),
            ErrorKind::InvalidValue | ErrorKind::ValueValidation => loader.get_args(
                "clap-error-invalid-value",
                HashMap::from([
                    ("value", context(error, ContextKind::InvalidValue)),
                    ("argument", context(error, ContextKind::InvalidArg)),
                ]),
            ),
            ErrorKind::UnknownArgument => loader.get_args(
                "clap-error-unknown-argument",
                HashMap::from([("argument", context(error, ContextKind::InvalidArg))]),
            ),
            ErrorKind::InvalidSubcommand => loader.get_args(
                "clap-error-unknown-subcommand",
                HashMap::from([("subcommand", context(error, ContextKind::InvalidSubcommand))]),
            ),
            ErrorKind::MissingSubcommand => loader.get_args(
                "clap-error-missing-subcommand",
                HashMap::from([("command", context(error, ContextKind::InvalidSubcommand))]),
            ),
            ErrorKind::ArgumentConflict => loader.get_args(
                "clap-error-conflict",
                HashMap::from([
                    ("argument", context(error, ContextKind::InvalidArg)),
                    ("prior", context(error, ContextKind::PriorArg)),
                ]),
            ),
            kind => kind.as_str().unwrap_or_default().to_owned(),
        };

        let mut styled = StyledStr::new();
        styled.push_str(&format!(
            "{error_style}{}:{error_style:#} {message}",
            loader.get("clap-error")
        ));

        if let Some(ContextValue::StyledStr(usage)) = error.get(ContextKind::Usage) {
            let usage = usage.to_string();
            let usage = usage.trim_start();
            let usage = usage.strip_prefix("Usage:").unwrap_or(usage).trim_start();

            styled.push_str(&format!(
                "\n\n{usage_style}{}:{usage_style:#} {usage}",
                loader.get("clap-usage")
            ));
        }

        styled.push_str(&format!(
            "\n\n{}\n",
            loader.get_args("clap-error-try-help", HashMap::from([("help", "--help")]))
        ));

        styled
    }
}

pub fn exit<L: CliLocale>(error: ClapError) -> ! {
    if is_localized(error.kind()) {
        error.apply::<LocalizedErrorFormatter<L>>().exit()
    }

    error.exit()
}

pub fn render<L: CliLocale>(error: ClapError) -> String {
    if is_localized(error.kind()) {
        return error.apply::<LocalizedErrorFormatter<L>>().to_string();
    }

    error.to_string()
}

fn is_localized(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::MissingRequiredArgument
            | ErrorKind::InvalidValue
            | ErrorKind::ValueValidation
            | ErrorKind::UnknownArgument
            | ErrorKind::InvalidSubcommand
            | ErrorKind::MissingSubcommand
            | ErrorKind::ArgumentConflict
    )
}

fn context<L: CliLocale>(error: &ClapError<LocalizedErrorFormatter<L>>, kind: ContextKind) -> String {
    error.get(kind).map(ContextValue::to_string).unwrap_or_default()
}
