// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use clap::CommandFactory;

use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed::select;
use i18n_embed::unic_langid::LanguageIdentifier;

use upac_locale::command::missing_keys;
use upac_locale::error::render;
use upac_locale::{CliLocale, try_parse_from};

use crate::Cli;

use super::EmbeddedAssets;

static ENGLISH_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| single_language_loader("en"));
static RUSSIAN_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| single_language_loader("ru"));

struct English;

impl CliLocale for English {
    fn loader() -> &'static FluentLanguageLoader {
        &ENGLISH_LOADER
    }
}

struct Russian;

impl CliLocale for Russian {
    fn loader() -> &'static FluentLanguageLoader {
        &RUSSIAN_LOADER
    }
}

fn single_language_loader(language: &str) -> FluentLanguageLoader {
    let language: LanguageIdentifier = language.parse().unwrap();
    let loader = FluentLanguageLoader::new("upac-setup-cli", language.clone());

    select(&loader, &EmbeddedAssets, &[language]).unwrap();
    loader.set_use_isolating(false);

    loader
}

fn parse_error<L: CliLocale>(arguments: &[&str]) -> String {
    render::<L>(try_parse_from::<Cli, L, _, _>(arguments.iter().copied()).err().unwrap())
}

#[test]
fn every_command_and_flag_has_an_english_translation() {
    assert_eq!(missing_keys::<English>(&Cli::command()), Vec::<String>::new());
}

#[test]
fn every_command_and_flag_has_a_russian_translation() {
    assert_eq!(missing_keys::<Russian>(&Cli::command()), Vec::<String>::new());
}

#[test]
fn a_missing_required_flag_is_reported_in_english() {
    let message = parse_error::<English>(&["up-sp", "partition", "create", "--device", "/dev/sda", "--size", "8G"]);

    assert!(
        message.starts_with("error: the following required arguments were not provided: --label <LABEL>"),
        "{message}"
    );
}

#[test]
fn a_missing_required_flag_is_reported_in_russian() {
    let message = parse_error::<Russian>(&[
        "up-sp",
        "bootstrap",
        "--esp-device",
        "/dev/sda1",
        "--deploy-device",
        "/dev/sda2",
    ]);

    assert!(
        message.starts_with("ошибка: не указаны обязательные аргументы: --source <SOURCE>"),
        "{message}"
    );
}

#[test]
fn an_unknown_partition_type_is_rejected_in_english() {
    let message = parse_error::<English>(&[
        "up-sp",
        "partition",
        "create",
        "--device",
        "/dev/sda",
        "--size",
        "8G",
        "--label",
        "root",
        "--type",
        "esp",
    ]);

    assert!(
        message.starts_with("error: invalid value 'esp' for '--type <KIND>'"),
        "{message}"
    );
}
