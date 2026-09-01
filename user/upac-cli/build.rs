// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::env::var;
use std::error::Error;
use std::fs::{read_to_string, write};
use std::path::Path;

use toml::{Value, from_str};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = var("CARGO_MANIFEST_DIR")?;
    let cli_toml = Path::new(&manifest_dir).join("../cli.toml");

    println!("cargo:rerun-if-changed={}", cli_toml.display());
    println!("cargo:rerun-if-env-changed=I18N_DIR");

    let i18n_dir = match var("I18N_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            let raw = read_to_string(&cli_toml)?;
            let config: Value = from_str(&raw)?;

            config
                .get("i18n_dir")
                .and_then(Value::as_str)
                .ok_or("cli.toml: missing i18n_dir")?
                .to_owned()
        }
    };

    let generated = format!("pub const I18N_DIR: &str = {i18n_dir:?};\n");

    let out = Path::new(&var("OUT_DIR")?).join("layout.rs");
    write(out, generated)?;

    Ok(())
}
