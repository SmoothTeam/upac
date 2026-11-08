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
    fn render_strings(values: &[Value], context: &str) -> Result<String, Box<dyn Error>> {
        let mut rendered = String::from("&[");

        for value in values {
            let value = value
                .as_str()
                .ok_or_else(|| format!("lib.toml: {context} entries must be strings"))?;
            rendered.push_str(&format!("{value:?}, "));
        }

        rendered.push(']');

        Ok(rendered)
    }

    let manifest_dir = var("CARGO_MANIFEST_DIR")?;
    let source = Path::new(&manifest_dir).join("lib.toml");

    println!("cargo:rerun-if-changed={}", source.display());

    let raw = read_to_string(&source)?;
    let config: Value = from_str(&raw)?;

    let format = config
        .get("format")
        .and_then(Value::as_str)
        .ok_or("lib.toml: format must be a string")?;

    let extensions = config
        .get("extensions")
        .and_then(Value::as_array)
        .ok_or("lib.toml: extensions must be an array")?;

    let mut generated = String::new();
    generated.push_str("pub mod manifest {\n");
    generated.push_str(&format!("    pub const FORMAT: &str = {format:?};\n"));
    generated.push_str(&format!(
        "    pub const EXTENSIONS: &[&str] = {};\n",
        render_strings(extensions, "extensions")?
    ));
    generated.push_str("}\n");

    let sections = config.as_table().ok_or("lib.toml: root must be a table")?;
    for (section, entries) in sections {
        let Some(entries) = entries.as_table() else {
            continue;
        };

        generated.push_str(&format!("pub mod {section} {{\n"));

        for (key, value) in entries {
            let key = key.to_uppercase();

            match value {
                Value::String(text) => generated.push_str(&format!("    pub const {key}: &str = {text:?};\n")),
                Value::Integer(number) => generated.push_str(&format!("    pub const {key}: u32 = {number};\n")),
                Value::Array(values) => generated.push_str(&format!(
                    "    pub const {key}: &[&str] = {};\n",
                    render_strings(values, &format!("{section}.{key}"))?
                )),
                _ => return Err(format!("lib.toml: {section}.{key} must be a string, integer or array").into()),
            }
        }

        generated.push_str("}\n");
    }

    let out = Path::new(&var("OUT_DIR")?).join("layout.rs");
    write(out, generated)?;

    Ok(())
}
