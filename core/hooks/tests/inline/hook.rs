// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{read_link, write};

use tempfile::{Builder, TempDir};

use super::super::error::HookError;
use super::super::primitive::Step;
use super::Hook;

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

#[test]
fn hook_parse_succeeds_for_trigger_map() {
    let hook = Hook::parse("[triggers]\ndeb = [\"postinst\"]\n").unwrap();

    assert_eq!(hook.triggers.get("deb").unwrap(), &vec!["postinst".to_owned()]);
}

#[test]
fn hook_parse_fails_when_no_trigger_at_all() {
    let result = Hook::parse("priority = 1\n");

    assert_eq!(result.unwrap_err(), HookError::NoTrigger);
}

#[test]
fn hook_parse_fails_on_malformed_toml() {
    let result = Hook::parse("not valid toml [[[");

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn touch_file_step_creates_missing_file_and_rollback_removes_it() {
    let dir = scratch_dir("touch-missing");
    let path = dir.path().join("marker");

    let mut hook = Hook::parse(&format!(
        "[triggers]\ndeb = [\"postinst\"]\n\n[[steps]]\ntype = \"touch_file\"\npath = {:?}\n",
        path
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute().unwrap();
    assert!(path.exists());

    step.rollback().unwrap();
    assert!(!path.exists());
}

#[test]
fn touch_file_step_leaves_preexisting_file_after_rollback() {
    let dir = scratch_dir("touch-existing");
    let path = dir.path().join("marker");
    write(&path, b"already here").unwrap();

    let mut hook = Hook::parse(&format!(
        "[triggers]\ndeb = [\"postinst\"]\n\n[[steps]]\ntype = \"touch_file\"\npath = {:?}\n",
        path
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute().unwrap();
    step.rollback().unwrap();

    assert!(path.exists());
}

#[test]
fn move_file_step_execute_and_rollback_round_trip() {
    let dir = scratch_dir("move-round-trip");
    let from = dir.path().join("a");
    let to = dir.path().join("b");
    write(&from, b"content").unwrap();

    let mut hook = Hook::parse(&format!(
        "[triggers]\ndeb = [\"postinst\"]\n\n[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        from, to
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute().unwrap();
    assert!(!from.exists());
    assert!(to.exists());

    step.rollback().unwrap();
    assert!(from.exists());
    assert!(!to.exists());
}

#[test]
fn create_symlink_step_execute_and_rollback() {
    let dir = scratch_dir("symlink");
    let target = dir.path().join("target");
    let link = dir.path().join("link");
    write(&target, b"content").unwrap();

    let mut hook = Hook::parse(&format!(
        "[triggers]\ndeb = [\"postinst\"]\n\n[[steps]]\ntype = \"create_symlink\"\ntarget = {:?}\nlink = {:?}\n",
        target, link
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute().unwrap();
    assert_eq!(read_link(&link).unwrap(), target);

    step.rollback().unwrap();
    assert!(!link.exists());
}

#[test]
fn rolling_back_a_hook_unwinds_its_steps_in_reverse_order() {
    let dir = scratch_dir("rollback-order");
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let c = dir.path().join("c");
    write(&a, b"content").unwrap();

    let mut hook = Hook::parse(&format!(
        concat!(
            "[triggers]\ndeb = [\"postinst\"]\n\n",
            "[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n\n",
            "[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        ),
        a, b, b, c
    ))
    .unwrap();

    hook.execute().unwrap();
    assert!(c.exists());

    hook.rollback().unwrap();

    assert!(a.exists());
    assert!(!b.exists());
    assert!(!c.exists());
}

#[test]
fn rolling_back_a_hook_that_never_ran_does_nothing() {
    let dir = scratch_dir("rollback-unexecuted");
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    write(&a, b"content").unwrap();

    let mut hook = Hook::parse(&format!(
        "[triggers]\ndeb = [\"postinst\"]\n\n[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        a, b
    ))
    .unwrap();

    hook.rollback().unwrap();

    assert!(a.exists());
    assert!(!b.exists());
}

#[test]
fn rolling_back_a_partly_run_hook_undoes_only_the_steps_that_ran() {
    let dir = scratch_dir("rollback-partial");
    let touched = dir.path().join("touched");
    let missing_from = dir.path().join("does-not-exist");
    let move_to = dir.path().join("move-to");

    let mut hook = Hook::parse(&format!(
        concat!(
            "critical = false\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
            "[[steps]]\ntype = \"touch_file\"\npath = {:?}\n\n",
            "[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        ),
        touched, missing_from, move_to
    ))
    .unwrap();

    hook.execute().unwrap();
    hook.rollback().unwrap();

    assert!(!touched.exists());
}

#[test]
fn executing_a_hook_runs_all_of_its_steps() {
    let dir = scratch_dir("run-advance");
    let a = dir.path().join("a");
    let b = dir.path().join("b");

    let mut hook = Hook::parse(&format!(
        concat!(
            "[triggers]\ndeb = [\"postinst\"]\n\n",
            "[[steps]]\ntype = \"touch_file\"\npath = {:?}\n\n",
            "[[steps]]\ntype = \"touch_file\"\npath = {:?}\n",
        ),
        a, b
    ))
    .unwrap();

    hook.execute().unwrap();

    assert!(a.exists());
    assert!(b.exists());
}

#[test]
fn a_failing_critical_hook_rolls_back_and_errors() {
    let dir = scratch_dir("run-critical-failure");
    let touched = dir.path().join("touched");
    let missing_from = dir.path().join("does-not-exist");
    let move_to = dir.path().join("move-to");

    let mut hook = Hook::parse(&format!(
        concat!(
            "critical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
            "[[steps]]\ntype = \"touch_file\"\npath = {:?}\n\n",
            "[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        ),
        touched, missing_from, move_to
    ))
    .unwrap();

    let result = hook.execute();

    assert!(result.is_err());
    assert!(!touched.exists(), "the already-executed step must be rolled back");
}

#[test]
fn a_failing_non_critical_hook_stops_without_an_error() {
    let dir = scratch_dir("run-non-critical-failure");
    let touched = dir.path().join("touched");
    let missing_from = dir.path().join("does-not-exist");
    let move_to = dir.path().join("move-to");

    let mut hook = Hook::parse(&format!(
        concat!(
            "critical = false\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
            "[[steps]]\ntype = \"touch_file\"\npath = {:?}\n\n",
            "[[steps]]\ntype = \"move_file\"\nfrom = {:?}\nto = {:?}\n",
        ),
        touched, missing_from, move_to
    ))
    .unwrap();

    hook.execute().unwrap();

    assert!(
        touched.exists(),
        "a non-critical failure must not roll back prior steps"
    );
}
