// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::lint_style::violation::Violation;

const RULE: &str = "extern-fn-position";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Extern,
    Pub,
    Private,
}

pub(super) fn check(path: &Path, contents: &str) -> Vec<Violation> {
    let tiers: Vec<(usize, Tier)> = contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| classify(line).map(|tier| (index, tier)))
        .collect();

    if !tiers.iter().any(|(_, tier)| *tier == Tier::Extern) {
        return Vec::new();
    }

    let mut violations = Vec::new();
    let mut highest_seen = Tier::Extern;

    for (index, tier) in tiers {
        if tier < highest_seen {
            violations.push(Violation {
                path: path.to_owned(),
                line: index + 1,
                rule: RULE,
                message: tier_violation_message(tier),
            });
        } else {
            highest_seen = tier;
        }
    }

    violations
}

fn tier_violation_message(tier: Tier) -> String {
    // `highest_seen` only ever grows, starting at `Extern` — so a violation can only ever be
    // triggered by an `Extern` or `Pub` line ranking below something already seen; `Private` is
    // the highest tier and can never trigger one.
    if tier == Tier::Extern {
        "extern \"C\" fn appears after a pub/private fn — extern fns go first".to_owned()
    } else {
        "pub fn appears after a private fn — pub fns go before private ones".to_owned()
    }
}

fn classify(line: &str) -> Option<Tier> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }

    let line = line.trim_end();

    if line.contains("extern \"C\" fn") {
        return Some(Tier::Extern);
    }
    if line.starts_with("pub ") && line.contains("fn ") {
        return Some(Tier::Pub);
    }
    if (line.starts_with("fn ") || line.starts_with("async fn ") || line.starts_with("unsafe fn "))
        && !line.starts_with("pub")
    {
        return Some(Tier::Private);
    }

    None
}
