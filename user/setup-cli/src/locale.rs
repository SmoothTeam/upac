// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use i18n_embed::DesktopLanguageRequester;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct EmbeddedAssets;

pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| fluent_language_loader!());

pub fn init() {
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _ = i18n_embed::select(&*LOADER, &EmbeddedAssets, &requested_languages);
}

#[cfg(test)]
pub(crate) fn init_for_test() {
    let english = "en".parse().unwrap();
    let _ = i18n_embed::select(&*LOADER, &EmbeddedAssets, &[english]);
}
