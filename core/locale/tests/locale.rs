// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::sync::LazyLock;

use clap::{Args, CommandFactory, Parser, Subcommand};

use i18n_embed::FileSystemAssets;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed::select;

use upac_locale::command::{localize, missing_keys};
use upac_locale::error::render;
use upac_locale::{CliLocale, try_parse_from};

static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let english = "en".parse().unwrap();
    let loader = FluentLanguageLoader::new("locale-test", "en".parse().unwrap());
    let assets = FileSystemAssets::try_new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/i18n")).unwrap();

    select(&loader, &assets, &[english]).unwrap();
    loader.set_use_isolating(false);

    loader
});

struct TestLocale;

impl CliLocale for TestLocale {
    fn loader() -> &'static FluentLanguageLoader {
        &LOADER
    }
}

#[derive(Parser)]
#[command(name = "demo")]
struct Cli {
    #[command(subcommand)]
    command: DemoCommand,
}

#[derive(Subcommand)]
enum DemoCommand {
    Create(CreateArgs),
}

#[derive(Args)]
struct CreateArgs {
    #[arg(long)]
    size: String,
    #[arg(long)]
    label: Option<String>,
    #[arg(long)]
    version: bool,
    #[arg(long, help_heading = "placement")]
    shelf: Option<String>,
}

#[test]
fn localize_sets_about_and_argument_help_from_their_keys() {
    let command = localize::<TestLocale>(Cli::command());
    let create = command.find_subcommand("create").unwrap();

    assert_eq!(command.get_about().unwrap().to_string(), "Demo tool");
    assert_eq!(create.get_about().unwrap().to_string(), "Create a thing");

    let size = create
        .get_arguments()
        .find(|arg| arg.get_id().as_str() == "size")
        .unwrap();
    assert_eq!(size.get_help().unwrap().to_string(), "Size of the thing");
}

#[test]
fn a_user_defined_version_flag_is_not_mistaken_for_the_builtin_one() {
    let command = localize::<TestLocale>(Cli::command());
    let create = command.find_subcommand("create").unwrap();

    let version = create
        .get_arguments()
        .find(|arg| arg.get_id().as_str() == "version")
        .unwrap();
    assert_eq!(version.get_help().unwrap().to_string(), "Show the version column");
}

#[test]
fn rendered_help_uses_the_localized_headings() {
    let mut command = localize::<TestLocale>(Cli::command());
    let help = command.find_subcommand_mut("create").unwrap().render_help().to_string();

    assert!(help.contains("USAGE: "), "{help}");
    assert!(help.contains("OPTS:"), "{help}");
    assert!(help.contains("Show help"), "{help}");
}

#[test]
fn a_custom_help_heading_is_localized_through_its_key() {
    let mut command = localize::<TestLocale>(Cli::command());
    let help = command.find_subcommand_mut("create").unwrap().render_help().to_string();

    assert!(help.contains("PLACE:"), "{help}");
    assert!(help.contains("Which shelf to use"), "{help}");
}

#[test]
fn parses_valid_arguments() {
    let cli = try_parse_from::<Cli, TestLocale, _, _>(["demo", "create", "--size", "8G"]).unwrap();

    let DemoCommand::Create(args) = cli.command;
    assert_eq!(args.size, "8G");
    assert_eq!(args.label, None);
    assert!(!args.version);
}

#[test]
fn missing_required_argument_renders_the_localized_error() {
    let error = try_parse_from::<Cli, TestLocale, _, _>(["demo", "create"])
        .err()
        .unwrap();
    let message = render::<TestLocale>(error);

    assert!(message.starts_with("ERR: missing: --size <SIZE>"), "{message}");
    assert!(message.contains("USAGE: demo create"), "{message}");
    assert!(message.ends_with("See --help\n"), "{message}");
}

#[test]
fn unknown_argument_renders_the_localized_error() {
    let error = try_parse_from::<Cli, TestLocale, _, _>(["demo", "create", "--size", "8G", "--bogus"])
        .err()
        .unwrap();
    let message = render::<TestLocale>(error);

    assert!(message.starts_with("ERR: unknown argument --bogus"), "{message}");
}

#[test]
fn missing_keys_reports_only_the_untranslated_argument() {
    assert_eq!(missing_keys::<TestLocale>(&Cli::command()), vec!["arg-create-label"]);
}
