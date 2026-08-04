// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use der::{Decode, Encode};
use ed25519_dalek::Signature;
use x509_cert::Certificate;

use crate::error::PkiError;

const SIGNATURE_LEN: usize = 64;
const LENGTH_PREFIX_LEN: usize = 4;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateKind {
    Hook = 0,
}

impl CertificateKind {
    fn from_u8(value: u8) -> Result<Self, PkiError> {
        match value {
            0 => Ok(CertificateKind::Hook),
            _ => Err(PkiError::Malformed),
        }
    }
}

pub struct HookSignature {
    pub certificate_kind: CertificateKind,
    pub certificate: Certificate,
    pub signature: Signature,
}

impl HookSignature {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PkiError> {
        let certificate_der = self.certificate.to_der()?;

        let mut bytes = Vec::with_capacity(1 + LENGTH_PREFIX_LEN + certificate_der.len() + SIGNATURE_LEN);

        bytes.push(self.certificate_kind as u8);

        bytes.extend_from_slice(&(certificate_der.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&certificate_der);

        bytes.extend_from_slice(&self.signature.to_bytes());

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PkiError> {
        let (&kind_byte, rest) = bytes.split_first().ok_or(PkiError::Malformed)?;
        let certificate_kind = CertificateKind::from_u8(kind_byte)?;

        if rest.len() < LENGTH_PREFIX_LEN {
            return Err(PkiError::Malformed);
        }
        let (length_bytes, rest) = rest.split_at(LENGTH_PREFIX_LEN);
        let certificate_len = u32::from_be_bytes(length_bytes.try_into()?) as usize;

        if rest.len() < certificate_len {
            return Err(PkiError::Malformed);
        }
        let (certificate_bytes, rest) = rest.split_at(certificate_len);
        let certificate = Certificate::from_der(certificate_bytes)?;

        if rest.len() != SIGNATURE_LEN {
            return Err(PkiError::Malformed);
        }
        let signature = Signature::try_from(rest).map_err(|_| PkiError::Malformed)?;

        Ok(HookSignature {
            certificate_kind,
            certificate,
            signature,
        })
    }
}
