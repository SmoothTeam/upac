// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::env::var;
use std::fs::{create_dir_all, read_dir};
use std::io::Result as IoResult;
use std::path::PathBuf;
use std::process::Command;

fn main() -> IoResult<()> {
    // Packager sets LOCALEDIR at build time; dev default is /usr/share/locale.
    let localedir = var("LOCALEDIR").unwrap_or_else(|_| "/usr/share/locale".to_owned());

    println!("cargo:rustc-env=LOCALEDIR={localedir}");
    println!("cargo:rerun-if-env-changed=LOCALEDIR");

    // Compile .po → .mo next to the output binary so `cargo run` works out of the box.
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let po_dir = manifest_dir.join("po");

    // target/{debug,release}/locale/
    let out_dir = PathBuf::from(var("OUT_DIR").expect("OUT_DIR not set"));
    let locale_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected OUT_DIR depth")
        .join("locale");

    if !po_dir.exists() {
        return Ok(());
    }

    for entry in read_dir(&po_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|extension| extension == "po") {
            let lang = path
                .file_stem()
                .expect("po file has no stem")
                .to_string_lossy()
                .into_owned();
            let mo_dir = locale_dir.join(&lang).join("LC_MESSAGES");
            create_dir_all(&mo_dir)?;

            let status = Command::new("msgfmt")
                .args([
                    "-o",
                    mo_dir.join("upac-setup.mo").to_str().expect("path is not valid UTF-8"),
                ])
                .arg(&path)
                .status();

            match status {
                Ok(status) if status.success() => {}
                Ok(status) => println!("cargo:warning=msgfmt exited with {status} for {lang}"),
                Err(error) => println!("cargo:warning=msgfmt not found ({error}), skipping {lang}"),
            }

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(())
}
