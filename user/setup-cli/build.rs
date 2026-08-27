// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::env::var;
use std::fs::read_dir;
use std::io::{Error as IoError, Result as IoResult};
use std::path::PathBuf;
use std::process::Command;

fn main() -> IoResult<()> {
    // Locale data is embedded into the binary (see main.rs's include_bytes! calls) so `up-sp`
    // stays self-contained in a bootstrap/rescue environment that has no system locale data of
    // its own — compile every po/*.po into a flat {lang}.mo directly under OUT_DIR.
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let po_dir = manifest_dir.join("po");
    let out_dir = PathBuf::from(var("OUT_DIR").expect("OUT_DIR not set"));

    for entry in read_dir(&po_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|extension| extension == "po") {
            let lang = path
                .file_stem()
                .expect("po file has no stem")
                .to_string_lossy()
                .into_owned();
            let mo_path = out_dir.join(format!("{lang}.mo"));

            let status = Command::new("msgfmt")
                .args(["-o", mo_path.to_str().expect("path is not valid UTF-8")])
                .arg(&path)
                .status()?;

            if !status.success() {
                return Err(IoError::other(format!("msgfmt exited with {status} for {lang}")));
            }

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(())
}
