// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::write;
use std::path::{Path, PathBuf};

use tempfile::{Builder, TempDir};

use upac_pki::generate::{Identity, SigningIdentity, generate_root, generate_signing_cert};
use upac_pki::signature::HookSignature;

use super::super::error::HookError;
use super::super::hook::Hook;
use super::HooksBuilder;

type FormatTriggers<'test> = (&'test str, &'test [&'test str]);

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

fn hook_toml(priority: i32, triggers: &[FormatTriggers<'_>]) -> String {
    let mut raw = format!("priority = {priority}\ncritical = true\nsteps = []\n\n[triggers]\n");

    for (format, names) in triggers {
        let quoted: Vec<String> = names.iter().map(|name| format!("{name:?}")).collect();
        raw.push_str(&format!("{format} = [{}]\n", quoted.join(", ")));
    }

    raw
}

fn builder_with(hooks: &[(&str, i32, &[FormatTriggers<'_>])]) -> HooksBuilder {
    let mut builder = HooksBuilder::new();

    for (name, priority, triggers) in hooks {
        builder.hooks.insert(
            (*name).to_owned(),
            Hook::parse(&hook_toml(*priority, triggers)).unwrap(),
        );
    }

    builder
}

#[test]
fn loading_accepts_a_signed_valid_hook_under_its_file_stem() {
    let hooks_dir = scratch_dir("load-valid");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-valid root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );

    let mut builder = HooksBuilder::new();
    builder.load(hooks_dir.path(), &cert_path).unwrap();

    assert!(builder.hooks.contains_key("postinst"));
}

#[test]
fn loading_skips_files_with_non_matching_extension() {
    let hooks_dir = scratch_dir("load-skip-extension");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-skip root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );
    write(hooks_dir.path().join("notes.txt"), b"not a hook").unwrap();

    let mut builder = HooksBuilder::new();
    builder.load(hooks_dir.path(), &cert_path).unwrap();

    assert_eq!(builder.hooks.len(), 1);
}

#[test]
fn loading_fails_when_signature_is_tampered() {
    let hooks_dir = scratch_dir("load-tampered");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-tampered root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );
    write(
        hooks_dir.path().join("postinst.hook"),
        hook_toml(0, &[("deb", &["prerm"])]),
    )
    .unwrap();

    let result = HooksBuilder::new().load(hooks_dir.path(), &cert_path);

    assert_eq!(result.unwrap_err(), HookError::InvalidSignature);
}

#[test]
fn loading_fails_when_root_cert_is_unrelated() {
    let hooks_dir = scratch_dir("load-unrelated-root");
    let (_, signing) = root_cert_file(hooks_dir.path(), "load-unrelated signing root");
    let (unrelated_cert_path, _) = root_cert_file(hooks_dir.path(), "load-unrelated other root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );

    let result = HooksBuilder::new().load(hooks_dir.path(), &unrelated_cert_path);

    assert_eq!(result.unwrap_err(), HookError::InvalidSignature);
}

#[test]
fn loading_fails_when_hooks_dir_is_missing() {
    let hooks_dir = scratch_dir("load-missing-dir").path().join("does-not-exist");
    let cert_dir = scratch_dir("load-missing-dir-cert");
    let (cert_path, _) = root_cert_file(cert_dir.path(), "load-missing-dir root");

    let result = HooksBuilder::new().load(&hooks_dir, &cert_path);

    assert!(matches!(result.unwrap_err(), HookError::Io(_)));
}

#[test]
fn loading_two_dirs_merges_their_hooks() {
    let first_dir = scratch_dir("load-merge-first");
    let second_dir = scratch_dir("load-merge-second");
    let (cert_path, signing) = root_cert_file(first_dir.path(), "load-merge root");
    write_signed_hook(
        first_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );
    write_signed_hook(
        second_dir.path(),
        "prerm",
        &hook_toml(0, &[("deb", &["prerm"])]),
        &signing,
    );

    let mut builder = HooksBuilder::new();
    builder.load(first_dir.path(), &cert_path).unwrap();
    builder.load(second_dir.path(), &cert_path).unwrap();

    assert_eq!(builder.hooks.len(), 2);
}

#[test]
fn loading_the_same_hook_name_twice_fails() {
    let first_dir = scratch_dir("load-duplicate-first");
    let second_dir = scratch_dir("load-duplicate-second");
    let (cert_path, signing) = root_cert_file(first_dir.path(), "load-duplicate root");
    write_signed_hook(
        first_dir.path(),
        "postinst",
        &hook_toml(0, &[("deb", &["postinst"])]),
        &signing,
    );
    write_signed_hook(
        second_dir.path(),
        "postinst",
        &hook_toml(1, &[("deb", &["postinst"])]),
        &signing,
    );

    let mut builder = HooksBuilder::new();
    builder.load(first_dir.path(), &cert_path).unwrap();
    let result = builder.load(second_dir.path(), &cert_path);

    assert_eq!(result.unwrap_err(), HookError::DuplicateHook("postinst".to_owned()));
}

#[test]
fn build_maps_a_trigger_to_its_only_hook() {
    let hooks = builder_with(&[("ldconfig", 0, &[("deb", &["postinst"])])])
        .build()
        .unwrap();

    assert_eq!(hooks.triggers["deb"]["postinst"], "ldconfig");
}

#[test]
fn build_keeps_formats_apart() {
    let hooks = builder_with(&[("ldconfig", 0, &[("rpm", &["posttrans"])])])
        .build()
        .unwrap();

    assert!(!hooks.triggers.contains_key("deb"));
}

#[test]
fn build_picks_the_higher_priority_hook() {
    let hooks = builder_with(&[
        ("high", 5, &[("deb", &["postinst"])]),
        ("low", 1, &[("deb", &["postinst"])]),
    ])
    .build()
    .unwrap();

    assert_eq!(hooks.triggers["deb"]["postinst"], "high");
}

#[test]
fn build_fails_on_a_priority_tie_naming_both_hooks() {
    let result = builder_with(&[
        ("first", 3, &[("deb", &["postinst"])]),
        ("second", 3, &[("deb", &["postinst"])]),
    ])
    .build();

    assert_eq!(
        result.unwrap_err(),
        HookError::TriggerConflict {
            format: "deb".to_owned(),
            name: "postinst".to_owned(),
            hooks: ("first".to_owned(), "second".to_owned()),
        }
    );
}

#[test]
fn build_lets_a_higher_priority_hook_settle_an_earlier_tie() {
    let hooks = builder_with(&[
        ("first", 3, &[("deb", &["postinst"])]),
        ("second", 3, &[("deb", &["postinst"])]),
        ("third", 5, &[("deb", &["postinst"])]),
    ])
    .build()
    .unwrap();

    assert_eq!(hooks.triggers["deb"]["postinst"], "third");
}

#[test]
fn build_keeps_distinct_names_independent() {
    let hooks = builder_with(&[("maintainer", 0, &[("deb", &["postinst", "postrm"])])])
        .build()
        .unwrap();

    let mut names: Vec<&str> = hooks.triggers["deb"].keys().map(String::as_str).collect();
    names.sort();

    assert_eq!(names, vec!["postinst", "postrm"]);
}
