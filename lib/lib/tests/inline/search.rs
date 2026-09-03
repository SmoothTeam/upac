// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::Search;

#[test]
fn substring_search_matches_regardless_of_case() {
    let search = Search::new("Upac", false).unwrap();

    assert!(search.is_match("this is upac"));
    assert!(search.is_match("this is UPAC"));
    assert!(!search.is_match("no match here"));
}

#[test]
fn regex_search_matches_using_the_pattern() {
    let search = Search::new(r"^up[a-z]+$", true).unwrap();

    assert!(search.is_match("upac"));
    assert!(!search.is_match("Upac"));
    assert!(!search.is_match("not-upac"));
}

#[test]
fn regex_search_rejects_an_invalid_pattern() {
    assert!(Search::new("(unclosed", true).is_err());
}
