// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_decoder_alpm::triggers;

#[test]
fn finds_no_triggers_in_empty_content() {
    let triggers = triggers::scan("");

    assert!(triggers.is_empty());
}

#[test]
fn finds_all_declared_lifecycle_functions_in_declaration_order() {
    let content = "post_remove() {\n    :\n}\n\npre_install() {\n    :\n}\n\npost_install ( ) {\n    :\n}\n";

    let triggers = triggers::scan(content);

    assert_eq!(triggers, vec!["pre_install", "post_install", "post_remove"]);
}

#[test]
fn ignores_names_that_are_not_function_declarations() {
    let content = "# pre_install is mentioned here but not declared\necho pre_install\n";

    let triggers = triggers::scan(content);

    assert!(triggers.is_empty());
}

#[test]
fn ignores_indented_occurrences() {
    let content = "post_install() {\n    pre_install()\n}\n";

    let triggers = triggers::scan(content);

    assert_eq!(triggers, vec!["post_install"]);
}
