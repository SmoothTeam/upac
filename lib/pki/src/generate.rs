// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use der::{Decode, Encode};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose,
    PKCS_ED25519,
};
use rustls_pki_types::CertificateDer;
use x509_cert::Certificate;

use crate::error::PkiError;

pub struct SerializedIdentity {
    pub key_der: Vec<u8>,
    pub certificate_der: Vec<u8>,
}

pub trait Identity: Sized {
    fn to_bytes(&self) -> Result<SerializedIdentity, PkiError>;
    fn from_bytes(serialized: &SerializedIdentity) -> Result<Self, PkiError>;
}

pub struct RootIdentity {
    pub issuer: Issuer<'static, KeyPair>,
    pub certificate: Certificate,
}

impl Identity for RootIdentity {
    fn to_bytes(&self) -> Result<SerializedIdentity, PkiError> {
        Ok(SerializedIdentity {
            key_der: self.issuer.key().serialize_der(),
            certificate_der: self.certificate.to_der()?,
        })
    }

    fn from_bytes(serialized: &SerializedIdentity) -> Result<Self, PkiError> {
        let key_pair = KeyPair::try_from(serialized.key_der.as_slice())?;
        let certificate_der = CertificateDer::from(serialized.certificate_der.as_slice());
        let issuer = Issuer::from_ca_cert_der(&certificate_der, key_pair)?;
        let certificate = Certificate::from_der(&serialized.certificate_der)?;

        Ok(RootIdentity { issuer, certificate })
    }
}

pub struct SigningIdentity {
    pub key_pair: KeyPair,
    pub certificate: Certificate,
}

impl Identity for SigningIdentity {
    fn to_bytes(&self) -> Result<SerializedIdentity, PkiError> {
        Ok(SerializedIdentity {
            key_der: self.key_pair.serialize_der(),
            certificate_der: self.certificate.to_der()?,
        })
    }

    fn from_bytes(serialized: &SerializedIdentity) -> Result<Self, PkiError> {
        let key_pair = KeyPair::try_from(serialized.key_der.as_slice())?;
        let certificate = Certificate::from_der(&serialized.certificate_der)?;

        Ok(SigningIdentity { key_pair, certificate })
    }
}

pub fn generate_root(common_name: &str) -> Result<RootIdentity, PkiError> {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, common_name);

    let mut params = CertificateParams::new(Vec::<String>::new())?;
    params.distinguished_name = distinguished_name;
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

    let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;
    let certificate_der = params.self_signed(&key_pair)?;
    let certificate = Certificate::from_der(certificate_der.der())?;

    let issuer = Issuer::new(params, key_pair);

    Ok(RootIdentity { issuer, certificate })
}

pub fn generate_signing_cert(common_name: &str, root: &RootIdentity) -> Result<SigningIdentity, PkiError> {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, common_name);

    let mut params = CertificateParams::new(Vec::<String>::new())?;
    params.distinguished_name = distinguished_name;
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];

    let key_pair = KeyPair::generate_for(&PKCS_ED25519)?;
    let certificate_der = params.signed_by(&key_pair, &root.issuer)?;
    let certificate = Certificate::from_der(certificate_der.der())?;

    Ok(SigningIdentity { key_pair, certificate })
}
