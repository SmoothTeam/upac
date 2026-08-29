// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::env::var;
use std::error::Error;
use std::fs::{read_to_string, write};
use std::path::Path;

use toml::{Value, from_str};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = var("CARGO_MANIFEST_DIR")?;

    let mut generated = String::new();
    generated.push_str(&generate_decoder_toml(&manifest_dir)?);
    generated.push_str(&generate_manifest_module(&manifest_dir)?);

    let out = Path::new(&var("OUT_DIR")?).join("layout.rs");
    write(out, generated)?;

    Ok(())
}

fn generate_decoder_toml(manifest_dir: &str) -> Result<String, Box<dyn Error>> {
    let source = Path::new(manifest_dir).join("../decoder.toml");

    println!("cargo:rerun-if-changed={}", source.display());

    let raw = read_to_string(&source)?;
    let config: Value = from_str(&raw)?;

    let section = "alpm";
    let entries = config
        .get(section)
        .and_then(Value::as_table)
        .ok_or_else(|| format!("decoder.toml: [{section}] must be a table"))?;

    let mut generated = String::new();
    generated.push_str(&format!("pub mod {section} {{\n"));

    for (key, value) in entries {
        let rendered = if let Some(value) = value.as_str() {
            format!("&str = {value:?}")
        } else if let Some(value) = value.as_integer() {
            format!("u32 = {value}")
        } else {
            return Err(format!("decoder.toml: {section}.{key} must be a string or integer").into());
        };

        generated.push_str(&format!("    pub const {}: {rendered};\n", key.to_uppercase()));
    }

    generated.push_str("}\n");

    Ok(generated)
}

/// Compiles this crate's own deployable manifest (`format`/`extensions`) into constants, so a
/// `builtin-alpm` build can dispatch by format without reading `upac-alpm.toml` from disk at
/// runtime — `library`/`mime` are runtime-deployment-only fields, not needed here.
fn generate_manifest_module(manifest_dir: &str) -> Result<String, Box<dyn Error>> {
    let source = Path::new(manifest_dir).join("upac-alpm.toml");

    println!("cargo:rerun-if-changed={}", source.display());

    let raw = read_to_string(&source)?;
    let config: Value = from_str(&raw)?;

    let format = config
        .get("format")
        .and_then(Value::as_str)
        .ok_or("upac-alpm.toml: format must be a string")?;

    let extensions = config
        .get("extensions")
        .and_then(Value::as_array)
        .ok_or("upac-alpm.toml: extensions must be an array")?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or("upac-alpm.toml: extensions entries must be strings")
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut generated = String::new();
    generated.push_str("pub mod manifest {\n");
    generated.push_str(&format!("    pub const FORMAT: &str = {format:?};\n"));
    generated.push_str("    pub const EXTENSIONS: &[&str] = &[\n");
    for extension in extensions {
        generated.push_str(&format!("        {extension:?},\n"));
    }
    generated.push_str("    ];\n");
    generated.push_str("}\n");

    Ok(generated)
}
