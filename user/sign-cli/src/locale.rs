// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{AssetsMultiplexor, DesktopLanguageRequester, FileSystemAssets, I18nAssets};

use rust_embed::RustEmbed;

use crate::layout::I18N_DIR;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct EmbeddedAssets;

pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| fluent_language_loader!());

pub fn init() {
    let mut sources: Vec<Box<dyn I18nAssets + Send + Sync>> = Vec::new();
    if let Ok(disk) = FileSystemAssets::try_new(I18N_DIR) {
        sources.push(Box::new(disk));
    }
    sources.push(Box::new(EmbeddedAssets));

    let assets = AssetsMultiplexor::new(sources);
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let _ = i18n_embed::select(&*LOADER, &assets, &requested_languages);
}
