// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::write;
use std::path::{Path, PathBuf};

use tempfile::{Builder, TempDir};

use upac_pki::generate::{Identity, SigningIdentity, generate_root, generate_signing_cert};
use upac_pki::signature::HookSignature;

use upac_types::decoder::{PackageTrigger, PackageTriggers, TriggerPosition};

use super::Hooks;
use super::error::HookError;

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

#[test]
fn loading_accepts_a_signed_valid_hook() {
    let hooks_dir = scratch_dir("load-valid");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-valid root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );

    let hooks = Hooks::load_from(hooks_dir.path(), &cert_path).unwrap();
    let packages = vec![deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert_eq!(
        hooks.matching(TriggerPosition::PostInstall, &packages).unwrap().len(),
        1
    );
}

#[test]
fn loading_skips_files_with_non_matching_extension() {
    let hooks_dir = scratch_dir("load-skip-extension");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-skip root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );
    write(hooks_dir.path().join("notes.txt"), b"not a hook").unwrap();

    let hooks = Hooks::load_from(hooks_dir.path(), &cert_path).unwrap();
    let packages = vec![deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert_eq!(
        hooks.matching(TriggerPosition::PostInstall, &packages).unwrap().len(),
        1
    );
}

#[test]
fn loading_fails_when_signature_is_tampered() {
    let hooks_dir = scratch_dir("load-tampered");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "load-tampered root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );

    write(
        hooks_dir.path().join("postinst.hook"),
        "[triggers]\ndeb = [\"prerm\"]\n",
    )
    .unwrap();

    let result = Hooks::load_from(hooks_dir.path(), &cert_path);

    assert_eq!(result.err().unwrap(), HookError::InvalidSignature);
}

#[test]
fn loading_fails_when_root_cert_is_unrelated() {
    let hooks_dir = scratch_dir("load-unrelated-root");
    let (_, signing) = root_cert_file(hooks_dir.path(), "load-unrelated signing root");
    let (unrelated_cert_path, _) = root_cert_file(hooks_dir.path(), "load-unrelated other root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );

    let result = Hooks::load_from(hooks_dir.path(), &unrelated_cert_path);

    assert_eq!(result.err().unwrap(), HookError::InvalidSignature);
}

#[test]
fn loading_fails_when_hooks_dir_is_missing() {
    let hooks_dir = scratch_dir("load-missing-dir").path().join("does-not-exist");
    let cert_dir = scratch_dir("load-missing-dir-cert");
    let (cert_path, _) = root_cert_file(cert_dir.path(), "load-missing-dir root");

    let result = Hooks::load_from(&hooks_dir, &cert_path);

    assert!(matches!(result.err().unwrap(), HookError::Io(_)));
}

#[test]
fn matching_ignores_package_triggers_of_other_positions() {
    let hooks_dir = scratch_dir("match-position");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "match-position root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );

    let hooks = Hooks::load_from(hooks_dir.path(), &cert_path).unwrap();
    let packages = vec![deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert!(
        hooks
            .matching(TriggerPosition::PreRemove, &packages)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn matching_runs_a_hook_once_however_many_packages_require_it() {
    let hooks_dir = scratch_dir("match-once");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "match-once root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );

    let hooks = Hooks::load_from(hooks_dir.path(), &cert_path).unwrap();
    let packages = vec![
        deb_package(&[(TriggerPosition::PostInstall, "postinst")]),
        deb_package(&[(TriggerPosition::PostInstall, "postinst")]),
    ];

    assert_eq!(
        hooks.matching(TriggerPosition::PostInstall, &packages).unwrap().len(),
        1
    );
}

#[test]
fn matching_skips_hooks_no_package_requires() {
    let hooks_dir = scratch_dir("match-unrequired");
    let (cert_path, signing) = root_cert_file(hooks_dir.path(), "match-unrequired root");
    write_signed_hook(
        hooks_dir.path(),
        "postinst",
        "[triggers]\ndeb = [\"postinst\"]\n",
        &signing,
    );
    write_signed_hook(hooks_dir.path(), "prerm", "[triggers]\ndeb = [\"prerm\"]\n", &signing);

    let hooks = Hooks::load_from(hooks_dir.path(), &cert_path).unwrap();
    let packages = vec![deb_package(&[(TriggerPosition::PostInstall, "postinst")])];

    assert_eq!(
        hooks.matching(TriggerPosition::PostInstall, &packages).unwrap().len(),
        1
    );
}
