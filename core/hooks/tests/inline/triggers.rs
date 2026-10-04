// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use super::{Hook, HookError, build_trigger_table};

fn hook_file(priority: i32, triggers: &[(&str, &[&str])]) -> Hook {
    let mut raw = format!("priority = {priority}\n\n[triggers]\n");
    for (format, names) in triggers {
        let quoted: Vec<String> = names.iter().map(|name| format!("{name:?}")).collect();
        raw.push_str(&format!("{format} = [{}]\n", quoted.join(", ")));
    }

    Hook::parse(&raw).unwrap()
}

#[test]
fn build_trigger_table_matches_single_hook() {
    let hooks = vec![hook_file(0, &[("deb", &["postinst"])])];

    let table = build_trigger_table(&hooks, "deb").unwrap();

    assert_eq!(table.len(), 1);
    assert_eq!(table.get("postinst"), Some(&0));
}

#[test]
fn build_trigger_table_ignores_other_formats() {
    let hooks = vec![hook_file(0, &[("rpm", &["posttrans"])])];

    let table = build_trigger_table(&hooks, "deb").unwrap();

    assert!(table.is_empty());
}

#[test]
fn build_trigger_table_picks_higher_priority_hook() {
    let hooks = vec![
        hook_file(1, &[("deb", &["postinst"])]),
        hook_file(5, &[("deb", &["postinst"])]),
    ];

    let table = build_trigger_table(&hooks, "deb").unwrap();

    assert_eq!(table.len(), 1);
    assert_eq!(table.get("postinst"), Some(&1));
}

#[test]
fn build_trigger_table_fails_on_priority_tie() {
    let hooks = vec![
        hook_file(3, &[("deb", &["postinst"])]),
        hook_file(3, &[("deb", &["postinst"])]),
    ];

    let result = build_trigger_table(&hooks, "deb");

    assert!(matches!(result, Err(HookError::TriggerConflict(name)) if name == "postinst"));
}

#[test]
fn build_trigger_table_keeps_distinct_names_independent() {
    let hooks = vec![hook_file(0, &[("deb", &["postinst", "postrm"])])];

    let table = build_trigger_table(&hooks, "deb").unwrap();
    let mut names: Vec<&str> = table.keys().copied().collect();
    names.sort();

    assert_eq!(names, vec!["postinst", "postrm"]);
}
