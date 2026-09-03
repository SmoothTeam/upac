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
    let manifest = var("CARGO_MANIFEST_DIR")?;
    let source = Path::new(&manifest).join("cli.toml");

    println!("cargo:rerun-if-changed={}", source.display());

    let raw = read_to_string(&source)?;
    let config: Value = from_str(&raw)?;

    let mut generated = String::new();

    let sections = config.as_table().ok_or("cli.toml: root must be a table")?;
    for (section, entries) in sections {
        generated.push_str(&format!("pub mod {section} {{\n"));

        let entries = entries
            .as_table()
            .ok_or_else(|| format!("cli.toml: [{section}] must be a table"))?;
        for (key, value) in entries {
            let key = key.to_uppercase();

            match value {
                Value::String(text) => generated.push_str(&format!("    pub const {key}: &str = {text:?};\n")),
                Value::Integer(number) => generated.push_str(&format!("    pub const {key}: u32 = {number};\n")),
                _ => return Err(format!("cli.toml: {section}.{key} must be a string or integer").into()),
            }
        }

        generated.push_str("}\n");
    }

    let out = Path::new(&var("OUT_DIR")?).join("layout.rs");
    write(out, generated)?;

    Ok(())
}
