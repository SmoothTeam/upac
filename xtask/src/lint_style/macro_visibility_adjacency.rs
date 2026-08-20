// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "macro-visibility-adjacency";

struct MacroBlock<'a> {
    name: &'a str,
    end_line: usize,
}

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let lines: Vec<&str> = contents.lines().collect();
    let macros = find_macro_blocks(&lines);

    let mut violations = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(name) = visibility_use_name(line) else {
            continue;
        };
        let Some(macro_block) = macros.iter().find(|block| block.name == name) else {
            continue;
        };

        if index != macro_block.end_line + 1 {
            violations.push(Violation {
                path: path.to_owned(),
                line: index + 1,
                rule: RULE,
                message: format!(
                    "`use {name};` re-exports macro_rules! {name}, but isn't the line immediately \
                     after its closing brace"
                ),
            });
        }
    }

    violations
}

fn find_macro_blocks<'a>(lines: &[&'a str]) -> Vec<MacroBlock<'a>> {
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let Some(name) = macro_name(lines[index]) else {
            index += 1;
            continue;
        };

        let mut depth = brace_delta(lines[index]);
        let mut end = index;
        while depth > 0 && end + 1 < lines.len() {
            end += 1;
            depth += brace_delta(lines[end]);
        }

        blocks.push(MacroBlock { name, end_line: end });
        index = end + 1;
    }

    blocks
}

fn brace_delta(line: &str) -> i32 {
    line.matches('{').count() as i32 - line.matches('}').count() as i32
}

fn macro_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("macro_rules! ")?;

    rest.split(|character: char| character == '{' || character.is_whitespace()).next()
}

fn visibility_use_name(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();

    let rest = trimmed
        .strip_prefix("pub(crate) use ")
        .or_else(|| trimmed.strip_prefix("pub(super) use "))
        .or_else(|| trimmed.strip_prefix("pub use "))?;

    rest.strip_suffix(';')
}
