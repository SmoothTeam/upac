// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "cargo-toml-package-order";

struct Field {
    line: usize,
    name: String,
    is_workspace: bool,
}

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let Some(fields) = package_fields(contents) else {
        return Vec::new();
    };

    let mut violations = Vec::new();

    if let Some(first) = fields.first() {
        if first.name != "name" {
            violations.push(violation(path, first.line, "`name` must be the first field in [package]"));
        }
    }

    if let Some(description) = fields.iter().find(|field| field.name == "description") {
        if fields.get(1).map(|field| field.name.as_str()) != Some("description") {
            violations.push(violation(
                path,
                description.line,
                "`description` must come right after `name`, before the `.workspace = true` fields",
            ));
        }
    }

    let mut seen_non_workspace = false;
    for field in fields.iter().filter(|field| field.name != "name" && field.name != "description") {
        if field.is_workspace {
            if seen_non_workspace {
                violations.push(violation(
                    path,
                    field.line,
                    &format!(
                        "`{}.workspace = true` appears after a non-workspace custom field — workspace fields go \
                         first",
                        field.name
                    ),
                ));
            }
        } else {
            seen_non_workspace = true;
        }
    }

    violations
}

fn package_fields(contents: &str) -> Option<Vec<Field>> {
    let lines: Vec<&str> = contents.lines().collect();
    let start = lines.iter().position(|line| line.trim() == "[package]")? + 1;

    let mut fields = Vec::new();
    for (offset, line) in lines[start..].iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some(equals) = trimmed.find('=') else {
            continue;
        };
        let key = trimmed[..equals].trim();

        let (name, is_workspace) = match key.strip_suffix(".workspace") {
            Some(base) => (base.to_owned(), true),
            None => (key.to_owned(), false),
        };

        fields.push(Field { line: start + offset, name, is_workspace });
    }

    Some(fields)
}

fn violation(path: &Path, line: usize, message: &str) -> Violation {
    Violation {
        path: path.to_owned(),
        line: line + 1,
        rule: RULE,
        message: message.to_owned(),
    }
}
