// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "toml-config-field-order";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Bool,
    Str,
    Number,
}

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let lines: Vec<&str> = contents.lines().collect();

    let mut violations = Vec::new();
    let mut block_start = 0;
    for (index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with('[') {
            violations.extend(check_block(path, &lines[block_start..index], block_start));
            block_start = index + 1;
        }
    }
    violations.extend(check_block(path, &lines[block_start..], block_start));

    violations
}

fn check_block(path: &Path, block: &[&str], block_start: usize) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut highest_kind = Kind::Bool;

    for (offset, line) in block.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some(equals) = trimmed.find('=') else {
            continue;
        };
        let key = trimmed[..equals].trim();
        let value = trimmed[equals + 1..].trim();

        let Some(kind) = classify(value) else {
            continue;
        };

        if kind < highest_kind {
            violations.push(Violation {
                path: path.to_owned(),
                line: block_start + offset + 1,
                rule: RULE,
                message: format!("`{key}` ({}) appears after a {} field — order is bool, string, number", kind_name(kind), kind_name(highest_kind)),
            });
        } else {
            highest_kind = kind;
        }
    }

    violations
}

fn classify(value: &str) -> Option<Kind> {
    if value == "true" || value == "false" {
        Some(Kind::Bool)
    } else if value.starts_with('"') {
        Some(Kind::Str)
    } else if value.starts_with(|character: char| character.is_ascii_digit() || character == '-' || character == '+') {
        Some(Kind::Number)
    } else {
        None
    }
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Bool => "bool",
        Kind::Str => "string",
        Kind::Number => "number",
    }
}
