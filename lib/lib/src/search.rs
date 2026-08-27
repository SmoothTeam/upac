// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use regex::Regex;

pub enum Search {
    Substring(String),
    Regex(Regex),
}

impl Search {
    pub fn new(pattern: &str, is_regex: bool) -> Result<Self, regex::Error> {
        if is_regex {
            Ok(Search::Regex(Regex::new(pattern)?))
        } else {
            Ok(Search::Substring(pattern.to_lowercase()))
        }
    }

    pub fn is_match(&self, haystack: &str) -> bool {
        match self {
            Search::Substring(needle) => haystack.to_lowercase().contains(needle),
            Search::Regex(regex) => regex.is_match(haystack),
        }
    }
}
