// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::env::temp_dir;
use std::fs::{create_dir_all, write};
use std::io::Result as IoResult;
use std::path::PathBuf;

// Must be kept in sync with the languages under po/*.po.
const CATALOGS: &[(&str, &[u8])] = &[
    ("en", include_bytes!(concat!(env!("OUT_DIR"), "/en.mo"))),
    ("ru", include_bytes!(concat!(env!("OUT_DIR"), "/ru.mo"))),
];

// Extracted rather than read from a system path, since up-sp may run in a bootstrap/rescue
// environment with no locale data of its own installed.
pub fn extract() -> IoResult<PathBuf> {
    let locale_dir = temp_dir().join("upac-setup-locale");

    for (lang, bytes) in CATALOGS {
        let lang_dir = locale_dir.join(lang).join("LC_MESSAGES");
        create_dir_all(&lang_dir)?;
        write(lang_dir.join("upac-setup.mo"), bytes)?;
    }

    Ok(locale_dir)
}
