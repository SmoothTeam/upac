// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{AssetsMultiplexor, DesktopLanguageRequester, FileSystemAssets, I18nAssets};

use rust_embed::RustEmbed;

use upac_types::settings::RuntimeSettings;

use crate::layout::I18N_DIR;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct EmbeddedAssets;

pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| fluent_language_loader!());

pub static SUBJECT_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| fluent_language_loader!());

fn assets() -> AssetsMultiplexor {
    let mut sources: Vec<Box<dyn I18nAssets + Send + Sync>> = Vec::new();
    if let Ok(disk) = FileSystemAssets::try_new(I18N_DIR) {
        sources.push(Box::new(disk));
    }
    sources.push(Box::new(EmbeddedAssets));

    AssetsMultiplexor::new(sources)
}

pub fn init() {
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let _ = i18n_embed::select(&*LOADER, &assets(), &requested_languages);

    let configured_language = RuntimeSettings::load()
        .locale
        .language
        .and_then(|language| language.parse().ok());

    let subject_languages = match configured_language {
        Some(language) => vec![language],
        None => requested_languages,
    };

    let _ = i18n_embed::select(&*SUBJECT_LOADER, &assets(), &subject_languages);
}

#[cfg(test)]
pub(crate) fn init_for_test() {
    let english = "en".parse().unwrap();
    let _ = i18n_embed::select(&*LOADER, &EmbeddedAssets, &[english]);
}
