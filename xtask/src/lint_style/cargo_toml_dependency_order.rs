// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "cargo-toml-dependency-order";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Upac,
    WorkspaceBracketed,
    Bracketed,
    Bare,
}

struct Entry {
    line: usize,
    name: String,
    tier: Tier,
    key_count: usize,
}

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let entries = dependency_entries(contents);

    let mut violations = Vec::new();
    let mut highest_tier = Tier::Upac;
    let mut last_bracketed_key_count = usize::MAX;

    for entry in &entries {
        if entry.tier < highest_tier {
            violations.push(violation(path, entry.line, &format!("`{}` is out of its expected group", entry.name)));
            continue;
        }

        if entry.tier == Tier::Bracketed && highest_tier == Tier::Bracketed && entry.key_count > last_bracketed_key_count
        {
            violations.push(violation(
                path,
                entry.line,
                &format!(
                    "`{}` ({} keys) appears after a bracketed dependency with fewer keys — sort by descending key \
                     count",
                    entry.name, entry.key_count
                ),
            ));
        }

        if entry.tier == Tier::Bracketed {
            last_bracketed_key_count = entry.key_count;
        }
        highest_tier = entry.tier;
    }

    violations
}

fn dependency_entries(contents: &str) -> Vec<Entry> {
    let lines: Vec<&str> = contents.lines().collect();
    let Some(start) = lines.iter().position(|line| line.trim() == "[dependencies]").map(|index| index + 1) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
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
        let name = trimmed[..equals].trim().to_owned();
        let value = trimmed[equals + 1..].trim();

        let tier = if name.starts_with("upac-") {
            Tier::Upac
        } else if value.starts_with('{') {
            if value.contains("workspace") {
                Tier::WorkspaceBracketed
            } else {
                Tier::Bracketed
            }
        } else {
            Tier::Bare
        };

        let key_count = if tier == Tier::Bracketed {
            trimmed.matches('=').count().saturating_sub(1)
        } else {
            0
        };

        entries.push(Entry { line: start + offset, name, tier, key_count });
    }

    entries
}

fn violation(path: &Path, line: usize, message: &str) -> Violation {
    Violation {
        path: path.to_owned(),
        line: line + 1,
        rule: RULE,
        message: message.to_owned(),
    }
}
