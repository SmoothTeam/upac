// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{File, create_dir, read_link, write};
use std::io::ErrorKind as IoErrorKind;
use std::os::fd::AsFd;
use std::os::unix::fs::symlink;
use std::path::Path;

use tempfile::{Builder, TempDir};

use super::super::error::HookError;
use super::super::primitive::{Primitive, Step};
use super::Hook;

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn root_open(dir: &TempDir) -> File {
    File::open(dir.path()).unwrap()
}

#[test]
fn hook_parse_succeeds_for_a_complete_hook() {
    let hook = Hook::parse("priority = 0\ncritical = true\nsteps = []\n\n[triggers]\ndeb = [\"postinst\"]\n").unwrap();

    assert_eq!(hook.triggers.get("deb").unwrap(), &vec!["postinst".to_owned()]);
}

#[test]
fn hook_parse_fails_when_no_trigger_at_all() {
    let result = Hook::parse("priority = 1\ncritical = true\nsteps = []\n\n[triggers]\n");

    assert_eq!(result.unwrap_err(), HookError::NoTrigger);
}

#[test]
fn hook_parse_fails_when_a_field_is_missing() {
    let result = Hook::parse("[triggers]\ndeb = [\"postinst\"]\n");

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn hook_parse_fails_on_an_unknown_field() {
    let result =
        Hook::parse("priority = 0\ncritical = true\nsteps = []\nunknown = 1\n\n[triggers]\ndeb = [\"postinst\"]\n");

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn hook_parse_fails_on_an_unknown_step_field() {
    let result = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/marker\"\nmode = 1\n",
    ));

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn hook_parse_fails_on_malformed_toml() {
    let result = Hook::parse("not valid toml [[[");

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn hook_parse_reads_a_run_step() {
    let hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"run\"\ncommand = \"ldconfig\"\nargs = [\"-X\"]\n",
    ))
    .unwrap();

    assert!(matches!(hook.steps.first(), Some(Primitive::Run(_))));
}

#[test]
fn hook_parse_fails_on_a_run_step_without_args() {
    let result = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"run\"\ncommand = \"ldconfig\"\n",
    ));

    assert_eq!(result.unwrap_err(), HookError::Parse);
}

#[test]
fn touch_file_step_creates_missing_file_and_rollback_removes_it() {
    let dir = scratch_dir("touch-missing");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/marker\"\n",
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute(root.as_fd()).unwrap();
    assert!(dir.path().join("marker").exists());

    step.rollback(root.as_fd()).unwrap();
    assert!(!dir.path().join("marker").exists());
}

#[test]
fn touch_file_step_leaves_preexisting_file_after_rollback() {
    let dir = scratch_dir("touch-existing");
    let root = root_open(&dir);
    write(dir.path().join("marker"), b"already here").unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/marker\"\n",
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute(root.as_fd()).unwrap();
    step.rollback(root.as_fd()).unwrap();

    assert!(dir.path().join("marker").exists());
}

#[test]
fn touch_file_step_rejects_a_relative_path() {
    let dir = scratch_dir("touch-relative");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"marker\"\n",
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    assert_eq!(step.execute(root.as_fd()).unwrap_err(), HookError::InvalidPath);
}

#[test]
fn absolute_symlinks_resolve_inside_the_root() {
    let dir = scratch_dir("touch-in-root");
    let root = root_open(&dir);
    create_dir(dir.path().join("inside")).unwrap();
    symlink("/inside", dir.path().join("escape")).unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/escape/marker\"\n",
    ))
    .unwrap();

    hook.execute(root.as_fd()).unwrap();

    assert!(dir.path().join("inside").join("marker").exists());
    assert!(!Path::new("/inside/marker").exists());
}

#[test]
fn move_file_step_execute_and_rollback_round_trip() {
    let dir = scratch_dir("move-round-trip");
    let root = root_open(&dir);
    let from = dir.path().join("a");
    let to = dir.path().join("b");
    write(&from, b"content").unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/a\"\nto = \"/b\"\n",
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute(root.as_fd()).unwrap();
    assert!(!from.exists());
    assert!(to.exists());

    step.rollback(root.as_fd()).unwrap();
    assert!(from.exists());
    assert!(!to.exists());
}

#[test]
fn create_symlink_step_execute_and_rollback() {
    let dir = scratch_dir("symlink");
    let root = root_open(&dir);
    let link = dir.path().join("link");
    write(dir.path().join("target"), b"content").unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"create_symlink\"\ntarget = \"/target\"\nlink = \"/link\"\n",
    ))
    .unwrap();
    let mut step = hook.steps.remove(0);

    step.execute(root.as_fd()).unwrap();
    assert_eq!(read_link(&link).unwrap(), Path::new("/target"));

    step.rollback(root.as_fd()).unwrap();
    assert!(!link.exists());
}

#[test]
fn rolling_back_a_hook_unwinds_its_steps_in_reverse_order() {
    let dir = scratch_dir("rollback-order");
    let root = root_open(&dir);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let c = dir.path().join("c");
    write(&a, b"content").unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/a\"\nto = \"/b\"\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/b\"\nto = \"/c\"\n",
    ))
    .unwrap();

    hook.execute(root.as_fd()).unwrap();
    assert!(c.exists());

    hook.rollback(root.as_fd()).unwrap();

    assert!(a.exists());
    assert!(!b.exists());
    assert!(!c.exists());
}

#[test]
fn rolling_back_a_hook_that_never_ran_does_nothing() {
    let dir = scratch_dir("rollback-unexecuted");
    let root = root_open(&dir);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    write(&a, b"content").unwrap();

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/a\"\nto = \"/b\"\n",
    ))
    .unwrap();

    hook.rollback(root.as_fd()).unwrap();

    assert!(a.exists());
    assert!(!b.exists());
}

#[test]
fn rolling_back_a_partly_run_hook_undoes_only_the_steps_that_ran() {
    let dir = scratch_dir("rollback-partial");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = false\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/touched\"\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/does-not-exist\"\nto = \"/move-to\"\n",
    ))
    .unwrap();

    hook.execute(root.as_fd()).unwrap();
    hook.rollback(root.as_fd()).unwrap();

    assert!(!dir.path().join("touched").exists());
}

#[test]
fn executing_a_hook_runs_all_of_its_steps() {
    let dir = scratch_dir("run-advance");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/a\"\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/b\"\n",
    ))
    .unwrap();

    hook.execute(root.as_fd()).unwrap();

    assert!(dir.path().join("a").exists());
    assert!(dir.path().join("b").exists());
}

#[test]
fn a_failing_critical_hook_rolls_back_and_errors() {
    let dir = scratch_dir("run-critical-failure");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = true\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/touched\"\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/does-not-exist\"\nto = \"/move-to\"\n",
    ))
    .unwrap();

    let result = hook.execute(root.as_fd());

    assert_eq!(result.unwrap_err(), HookError::Io(IoErrorKind::NotFound));
    assert!(!dir.path().join("touched").exists());
}

#[test]
fn a_failing_non_critical_hook_stops_without_an_error() {
    let dir = scratch_dir("run-non-critical-failure");
    let root = root_open(&dir);

    let mut hook = Hook::parse(concat!(
        "priority = 0\ncritical = false\n\n[triggers]\ndeb = [\"postinst\"]\n\n",
        "[[steps]]\ntype = \"touch_file\"\npath = \"/touched\"\n\n",
        "[[steps]]\ntype = \"move_file\"\nfrom = \"/does-not-exist\"\nto = \"/move-to\"\n",
    ))
    .unwrap();

    hook.execute(root.as_fd()).unwrap();

    assert!(dir.path().join("touched").exists());
}
