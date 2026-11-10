// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::cell::RefCell;
use std::fs::{File, write};
use std::io::ErrorKind as IoErrorKind;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

use tempfile::{Builder, TempDir};

use upac_types::CancelToken;
use upac_types::decoder::{PackageTrigger, PackageTriggers, TriggerPosition};

use upac_pki::generate::{Identity, SigningIdentity, generate_root, generate_signing_cert};
use upac_pki::signature::HookSignature;

use upac_hooks::builder::HooksBuilder;
use upac_hooks::error::HookError;
use upac_hooks::{HookExecution, Hooks};

fn scratch_dir(name: &str) -> TempDir {
    Builder::new().prefix(name).tempdir().unwrap()
}

fn root_cert_file(dir: &Path, common_name: &str) -> (PathBuf, SigningIdentity) {
    let root = generate_root(common_name).unwrap();
    let signing = generate_signing_cert(&format!("{common_name} signer"), &root).unwrap();

    let cert_path = dir.join("root.der");
    write(&cert_path, root.to_bytes().unwrap().certificate_der).unwrap();

    (cert_path, signing)
}

fn write_signed_hook(dir: &Path, name: &str, hook_toml: &str, signing: &SigningIdentity) {
    write(dir.join(format!("{name}.hook")), hook_toml).unwrap();

    let signature = HookSignature::sign(hook_toml.as_bytes(), signing).unwrap();
    write(dir.join(format!("{name}.hook.sig")), signature.to_bytes().unwrap()).unwrap();
}

fn hooks_load(name: &str, hook_files: &[(&str, &str)]) -> Hooks {
    let hooks_dir = scratch_dir(name);
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), name);

    for (hook_name, hook_toml) in hook_files {
        write_signed_hook(hooks_dir.path(), hook_name, hook_toml, &signing);
    }

    let mut builder = HooksBuilder::new();
    builder.load(hooks_dir.path(), &cert_path).unwrap();

    builder.build().unwrap()
}

fn deb_package(triggers: &[(TriggerPosition, &str)]) -> PackageTriggers {
    PackageTriggers {
        format: "deb".to_owned(),
        triggers: triggers
            .iter()
            .map(|(position, name)| PackageTrigger {
                position: *position,
                name: (*name).to_owned(),
            })
            .collect(),
    }
}

fn touching_hook(priority: i32, trigger_name: &str, path: &str) -> String {
    format!(
        "priority = {priority}\ncritical = true\n\n[triggers]\ndeb = [{trigger_name:?}]\n\n[[steps]]\ntype = \"touch_file\"\npath = {path:?}\n"
    )
}

#[test]
fn executing_runs_the_matched_hooks_in_order_and_reports_progress() {
    let hooks = hooks_load(
        "execute-order",
        &[
            ("late", &touching_hook(1, "postinst", "/late")),
            ("early", &touching_hook(9, "triggered", "/early")),
        ],
    );
    let root_dir = scratch_dir("execute-order-root");
    let root = File::open(root_dir.path()).unwrap();
    let packages = [deb_package(&[
        (TriggerPosition::PostInstall, "postinst"),
        (TriggerPosition::PostInstall, "triggered"),
    ])];
    let reported = RefCell::new(Vec::new());

    let mut execution = HookExecution::new();
    execution
        .execute(
            &hooks,
            TriggerPosition::PostInstall,
            &packages,
            root.as_fd(),
            &CancelToken::new(),
            &|hook_name, done, total| reported.borrow_mut().push((hook_name.map(str::to_owned), done, total)),
        )
        .unwrap();

    assert!(root_dir.path().join("early").exists());
    assert!(root_dir.path().join("late").exists());
    assert_eq!(
        reported.into_inner(),
        vec![(Some("early".to_owned()), 1, 2), (Some("late".to_owned()), 2, 2)]
    );
}

#[test]
fn rolling_back_undoes_every_executed_hook() {
    let hooks = hooks_load(
        "execute-rollback",
        &[
            ("first", &touching_hook(1, "postinst", "/first")),
            ("second", &touching_hook(1, "prerm", "/second")),
        ],
    );
    let root_dir = scratch_dir("execute-rollback-root");
    let root = File::open(root_dir.path()).unwrap();
    let cancel = CancelToken::new();

    let mut execution = HookExecution::new();
    execution
        .execute(
            &hooks,
            TriggerPosition::PostInstall,
            &[deb_package(&[(TriggerPosition::PostInstall, "postinst")])],
            root.as_fd(),
            &cancel,
            &|_, _, _| {},
        )
        .unwrap();
    execution
        .execute(
            &hooks,
            TriggerPosition::PreRemove,
            &[deb_package(&[(TriggerPosition::PreRemove, "prerm")])],
            root.as_fd(),
            &cancel,
            &|_, _, _| {},
        )
        .unwrap();

    execution.rollback(root.as_fd()).unwrap();

    assert!(!root_dir.path().join("first").exists());
    assert!(!root_dir.path().join("second").exists());
}

#[test]
fn a_failed_critical_hook_leaves_the_earlier_ones_to_rollback() {
    let failing = "priority = 1\ncritical = true\n\n[triggers]\ndeb = [\"broken\"]\n\n[[steps]]\ntype = \"move_file\"\nfrom = \"/does-not-exist\"\nto = \"/moved\"\n";
    let hooks = hooks_load(
        "execute-failure",
        &[
            ("working", &touching_hook(9, "postinst", "/working")),
            ("failing", failing),
        ],
    );
    let root_dir = scratch_dir("execute-failure-root");
    let root = File::open(root_dir.path()).unwrap();
    let packages = [deb_package(&[
        (TriggerPosition::PostInstall, "postinst"),
        (TriggerPosition::PostInstall, "broken"),
    ])];

    let mut execution = HookExecution::new();
    let result = execution.execute(
        &hooks,
        TriggerPosition::PostInstall,
        &packages,
        root.as_fd(),
        &CancelToken::new(),
        &|_, _, _| {},
    );

    assert_eq!(result.unwrap_err(), HookError::Io(IoErrorKind::NotFound));
    assert!(root_dir.path().join("working").exists());

    execution.rollback(root.as_fd()).unwrap();

    assert!(!root_dir.path().join("working").exists());
}

fn executed_names(hooks: &Hooks, position: TriggerPosition, packages: &[PackageTriggers]) -> Vec<String> {
    let root_dir = scratch_dir("execute-names-root");
    let root = File::open(root_dir.path()).unwrap();
    let reported = RefCell::new(Vec::new());

    HookExecution::new()
        .execute(
            hooks,
            position,
            packages,
            root.as_fd(),
            &CancelToken::new(),
            &|hook_name, _, _| reported.borrow_mut().extend(hook_name.map(str::to_owned)),
        )
        .unwrap();

    reported.into_inner()
}

#[test]
fn executing_ignores_package_triggers_of_other_positions() {
    let hooks = hooks_load(
        "execute-position",
        &[("ldconfig", &touching_hook(1, "postinst", "/marker"))],
    );
    let packages = [deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert!(executed_names(&hooks, TriggerPosition::PreRemove, &packages).is_empty());
}

#[test]
fn executing_ignores_packages_of_other_formats() {
    let hooks = hooks_load(
        "execute-format",
        &[("ldconfig", &touching_hook(1, "postinst", "/marker"))],
    );
    let packages = [PackageTriggers {
        format: "rpm".to_owned(),
        triggers: vec![PackageTrigger {
            position: TriggerPosition::PostInstall,
            name: "postinst".to_owned(),
        }],
    }];

    assert!(executed_names(&hooks, TriggerPosition::PostInstall, &packages).is_empty());
}

#[test]
fn executing_runs_a_hook_once_however_many_packages_require_it() {
    let hook = "priority = 1\ncritical = true\nsteps = []\n\n[triggers]\ndeb = [\"postinst\", \"configure\"]\n";
    let hooks = hooks_load("execute-once", &[("maintainer", hook)]);
    let packages = [
        deb_package(&[(TriggerPosition::PostInstall, "postinst")]),
        deb_package(&[(TriggerPosition::PostInstall, "configure")]),
        deb_package(&[(TriggerPosition::PostInstall, "postinst")]),
    ];

    assert_eq!(
        executed_names(&hooks, TriggerPosition::PostInstall, &packages),
        vec!["maintainer"]
    );
}

#[test]
fn executing_skips_hooks_no_package_requires() {
    let hooks = hooks_load(
        "execute-unrequired",
        &[
            ("ldconfig", &touching_hook(1, "postinst", "/ldconfig")),
            ("cleanup", &touching_hook(1, "prerm", "/cleanup")),
        ],
    );
    let packages = [deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert_eq!(
        executed_names(&hooks, TriggerPosition::PostInstall, &packages),
        vec!["ldconfig"]
    );
}

#[test]
fn executing_orders_equal_priorities_by_name() {
    let hooks = hooks_load(
        "execute-name-order",
        &[
            ("beta", &touching_hook(1, "first", "/beta")),
            ("alpha", &touching_hook(1, "second", "/alpha")),
        ],
    );
    let packages = [deb_package(&[
        (TriggerPosition::PostInstall, "first"),
        (TriggerPosition::PostInstall, "second"),
    ])];

    assert_eq!(
        executed_names(&hooks, TriggerPosition::PostInstall, &packages),
        vec!["alpha", "beta"]
    );
}

#[test]
fn executing_stops_when_cancelled() {
    let hooks = hooks_load(
        "execute-cancel",
        &[("ldconfig", &touching_hook(1, "postinst", "/marker"))],
    );
    let root_dir = scratch_dir("execute-cancel-root");
    let root = File::open(root_dir.path()).unwrap();
    let cancel = CancelToken::new();
    cancel.cancel();

    let result = HookExecution::new().execute(
        &hooks,
        TriggerPosition::PostInstall,
        &[deb_package(&[(TriggerPosition::PostInstall, "postinst")])],
        root.as_fd(),
        &cancel,
        &|_, _, _| {},
    );

    assert_eq!(result.unwrap_err(), HookError::Cancelled);
    assert!(!root_dir.path().join("marker").exists());
}
