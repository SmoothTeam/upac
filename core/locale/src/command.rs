// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::mem::take;

use clap::builder::StyledStr;
use clap::builder::styling::Styles;
use clap::{Arg, Command};

use i18n_embed::fluent::FluentLanguageLoader;

use super::CliLocale;

pub const SHARED_KEYS: &[&str] = &[
    "clap-usage",
    "clap-options",
    "clap-arguments",
    "clap-commands",
    "clap-help",
    "clap-version",
    "clap-help-subcommand",
    "clap-error",
    "clap-error-try-help",
    "clap-error-missing-argument",
    "clap-error-invalid-value",
    "clap-error-unknown-argument",
    "clap-error-unknown-subcommand",
    "clap-error-missing-subcommand",
    "clap-error-conflict",
];

const HELP_ARG: &str = "help";
const VERSION_ARG: &str = "version";
const HELP_SUBCOMMAND: &str = "help";

pub fn localize<L: CliLocale>(mut command: Command) -> Command {
    command.build();

    localize_command(&mut command, &mut Vec::new(), L::loader());

    command
}

pub fn missing_keys<L: CliLocale>(command: &Command) -> Vec<String> {
    let mut command = command.clone();
    command.build();

    let mut keys: Vec<String> = SHARED_KEYS.iter().map(|key| (*key).to_owned()).collect();
    collect_keys(&command, &mut Vec::new(), &mut keys);

    let loader = L::loader();
    keys.retain(|key| !loader.has(key));
    keys.sort();
    keys.dedup();

    keys
}

fn localize_command(command: &mut Command, path: &mut Vec<String>, loader: &FluentLanguageLoader) {
    let about = translated(loader, &about_key(path));

    let mut localized = take(command)
        .help_template(help_template(loader))
        .subcommand_help_heading(loader.get("clap-commands"))
        .mut_args(|arg| localize_arg(arg, path, loader));

    if let Some(about) = about {
        localized = localized.about(about);
    }

    *command = localized;

    for subcommand in command.get_subcommands_mut() {
        if subcommand.get_name() == HELP_SUBCOMMAND {
            *subcommand = take(subcommand).about(loader.get("clap-help-subcommand"));
            continue;
        }

        path.push(subcommand.get_name().to_owned());
        localize_command(subcommand, path, loader);
        path.pop();
    }
}

fn localize_arg(arg: Arg, path: &[String], loader: &FluentLanguageLoader) -> Arg {
    let key = match arg.get_id().as_str() {
        HELP_ARG => "clap-help".to_owned(),
        VERSION_ARG => "clap-version".to_owned(),
        _ => arg_key(&arg, path),
    };

    let heading = if arg.is_positional() {
        "clap-arguments"
    } else {
        "clap-options"
    };

    let arg = match arg.get_help_heading() {
        Some(_) => arg,
        None => arg.help_heading(loader.get(heading)),
    };

    match translated(loader, &key) {
        Some(help) => arg.help(help),
        None => arg,
    }
}

fn collect_keys(command: &Command, path: &mut Vec<String>, keys: &mut Vec<String>) {
    keys.push(about_key(path));

    for arg in command.get_arguments() {
        let id = arg.get_id().as_str();
        if id == HELP_ARG || id == VERSION_ARG || arg.is_hide_set() {
            continue;
        }

        keys.push(arg_key(arg, path));
    }

    for subcommand in command.get_subcommands() {
        if subcommand.get_name() == HELP_SUBCOMMAND {
            continue;
        }

        path.push(subcommand.get_name().to_owned());
        collect_keys(subcommand, path, keys);
        path.pop();
    }
}

fn about_key(path: &[String]) -> String {
    if path.is_empty() {
        return "about".to_owned();
    }

    format!("about-{}", path.join("-"))
}

fn arg_key(arg: &Arg, path: &[String]) -> String {
    let name = arg
        .get_long()
        .unwrap_or_else(|| arg.get_id().as_str())
        .replace('_', "-");

    if arg.is_global_set() || path.is_empty() {
        return format!("arg-{name}");
    }

    format!("arg-{}-{name}", path.join("-"))
}

fn help_template(loader: &FluentLanguageLoader) -> StyledStr {
    let usage_style = Styles::default().get_usage().to_owned();
    let usage_heading = loader.get("clap-usage");

    StyledStr::from(format!(
        "{{before-help}}{{about-with-newline}}\n{usage_style}{usage_heading}:{usage_style:#} {{usage}}\n\n{{all-args}}{{after-help}}"
    ))
}

fn translated(loader: &FluentLanguageLoader, key: &str) -> Option<String> {
    loader.has(key).then(|| loader.get(key))
}
