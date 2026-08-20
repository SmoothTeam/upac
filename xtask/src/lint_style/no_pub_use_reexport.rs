// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashSet;
use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "no-pub-use-reexport";

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let public_modules = public_module_names(contents);

    let mut violations = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let Some(module) = reexported_module(line) else {
            continue;
        };

        if public_modules.contains(module) {
            violations.push(Violation {
                path: path.to_owned(),
                line: index + 1,
                rule: RULE,
                message: format!(
                    "pub use self::{module}::... re-exports an already-`pub mod` submodule — access \
                     through the full module path instead"
                ),
            });
        }
    }

    violations
}

fn public_module_names(contents: &str) -> HashSet<&str> {
    contents
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("pub mod "))
        .filter_map(|rest| {
            rest.split(|character: char| character == ';' || character.is_whitespace())
                .next()
        })
        .collect()
}

fn reexported_module(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("pub use self::")?;

    rest.split("::").next()
}
