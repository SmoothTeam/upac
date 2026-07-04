use std::env::var;
use std::fs::{read_to_string, write};
use std::path::Path;

use toml::{Value, from_str};

fn main() {
    let manifest = var("CARGO_MANIFEST_DIR").unwrap();
    let source = Path::new(&manifest).join("Lib.toml");

    println!("cargo:rerun-if-changed={}", source.display());

    let raw = read_to_string(&source).unwrap();
    let config: Value = from_str(&raw).unwrap();

    let mut generated = String::new();

    for (section, entries) in config.as_table().unwrap() {
        generated.push_str(&format!("pub mod {section} {{\n"));
        for (key, value) in entries.as_table().unwrap() {
            generated.push_str(&format!(
                "    pub const {}: &str = {:?};\n",
                key.to_uppercase(),
                value.as_str().unwrap(),
            ));
        }
        generated.push_str("}\n");
    }

    let out = Path::new(&var("OUT_DIR").unwrap()).join("layout.rs");

    write(out, generated).unwrap();
}
