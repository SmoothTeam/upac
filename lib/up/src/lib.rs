// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use regex::Regex;

use upac_types::error::ErrorKind;

pub mod export;

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}
mod mutated;
mod unmutated;

#[cfg(test)]
#[path = "../tests/inline/search.rs"]
mod tests;

pub(crate) enum Search {
    Substring(String),
    Regex(Regex),
}

impl Search {
    pub fn new(pattern: &str, is_regex: bool) -> Result<Self, ErrorKind> {
        if is_regex {
            Regex::new(pattern)
                .map(Search::Regex)
                .map_err(|_| ErrorKind::InvalidEntry)
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
