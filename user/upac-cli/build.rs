// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Packager sets LOCALEDIR at build time; dev default is /usr/share/locale.
    let localedir = std::env::var("LOCALEDIR").unwrap_or_else(|_| "/usr/share/locale".to_owned());
    println!("cargo:rustc-env=LOCALEDIR={localedir}");
    println!("cargo:rerun-if-env-changed=LOCALEDIR");

    // Compile .po → .mo next to the output binary so `cargo run` works out of the box.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let po_dir = manifest_dir.join("po");

    // target/{debug,release}/locale/
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let locale_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected OUT_DIR depth")
        .join("locale");

    if !po_dir.exists() {
        return;
    }

    for entry in std::fs::read_dir(&po_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "po") {
            let lang = path.file_stem().unwrap().to_string_lossy().into_owned();
            let mo_dir = locale_dir.join(&lang).join("LC_MESSAGES");
            std::fs::create_dir_all(&mo_dir).unwrap();

            let status = Command::new("msgfmt")
                .args(["-o", mo_dir.join("upac.mo").to_str().unwrap()])
                .arg(&path)
                .status();

            match status {
                Ok(s) if s.success() => {}
                Ok(s) => println!("cargo:warning=msgfmt exited with {s} for {lang}"),
                Err(e) => println!("cargo:warning=msgfmt not found ({e}), skipping {lang}"),
            }

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
