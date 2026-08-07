// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use der::{Decode, Encode};
use x509_cert::Certificate;

use upac_pki::error::PkiError;
use upac_pki::generate::{Identity, SigningIdentity, generate_root, generate_signing_cert};
use upac_pki::signature::{HookSignature, RootCertificate};

fn root_certificate_of(certificate: &Certificate) -> RootCertificate {
    RootCertificate(Certificate::from_der(&certificate.to_der().unwrap()).unwrap())
}

fn signing_identity() -> (SigningIdentity, RootCertificate) {
    let root = generate_root("upac test root").unwrap();
    let signing = generate_signing_cert("upac test signer", &root).unwrap();

    (signing, root_certificate_of(&root.certificate))
}

#[test]
fn sign_and_verify_round_trip_succeeds() {
    let (signing, root_certificate) = signing_identity();
    let hook_bytes = b"#!/bin/sh\necho hook";

    let signature = HookSignature::sign(hook_bytes, &signing).unwrap();

    assert!(signature.verify(hook_bytes, &root_certificate).is_ok());
}

#[test]
fn verify_fails_with_tampered_hook_bytes() {
    let (signing, root_certificate) = signing_identity();
    let signature = HookSignature::sign(b"#!/bin/sh\necho hook", &signing).unwrap();

    let result = signature.verify(b"#!/bin/sh\necho tampered", &root_certificate);

    assert_eq!(result, Err(PkiError::InvalidSignature));
}

#[test]
fn verify_fails_with_unrelated_root() {
    let (signing, _) = signing_identity();
    let unrelated_root = generate_root("unrelated root").unwrap();
    let hook_bytes = b"#!/bin/sh\necho hook";

    let signature = HookSignature::sign(hook_bytes, &signing).unwrap();
    let result = signature.verify(hook_bytes, &root_certificate_of(&unrelated_root.certificate));

    assert_eq!(result, Err(PkiError::InvalidSignature));
}

#[test]
fn hook_signature_bytes_round_trip() {
    let (signing, root_certificate) = signing_identity();
    let hook_bytes = b"#!/bin/sh\necho hook";

    let signature = HookSignature::sign(hook_bytes, &signing).unwrap();
    let restored = HookSignature::from_bytes(&signature.to_bytes().unwrap()).unwrap();

    assert!(restored.verify(hook_bytes, &root_certificate).is_ok());
}

#[test]
fn hook_signature_from_bytes_rejects_malformed_input() {
    assert!(matches!(HookSignature::from_bytes(&[]), Err(PkiError::Malformed)));
    assert!(matches!(
        HookSignature::from_bytes(&[0, 1, 2]),
        Err(PkiError::Malformed)
    ));
    assert!(matches!(
        HookSignature::from_bytes(&[7, 0, 0, 0, 0]),
        Err(PkiError::Malformed)
    ));
}

#[test]
fn signing_identity_bytes_round_trip() {
    let (signing, root_certificate) = signing_identity();
    let hook_bytes = b"#!/bin/sh\necho hook";

    let restored = SigningIdentity::from_bytes(&signing.to_bytes().unwrap()).unwrap();
    let signature = HookSignature::sign(hook_bytes, &restored).unwrap();

    assert!(signature.verify(hook_bytes, &root_certificate).is_ok());
}
