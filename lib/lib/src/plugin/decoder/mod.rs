// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::mem::{MaybeUninit, size_of};

use libloading::Library;

use upac_abi::ABI_VERSION;
use upac_abi::decoder::{
    AbiVersionFn, CDecodeRequest, CDecodeResponse, CTriggerMatches, CTriggerTable, DecodeFn, MatchTriggersFn,
};
use upac_abi::hook::CancelToken;
use upac_abi::types::{CBorrowed, CSlice};

use crate::plugin::decoder::error::DecoderError;
use crate::types::{Dependency, PackageMeta};

pub mod error;

unsafe fn load_symbol<T: Copy>(library: &Library, name: &str) -> Result<T, DecoderError> {
    unsafe { library.get::<T>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|_| DecoderError::Symbol)
}

pub struct DecodedPackage {
    pub meta: PackageMeta,
    pub dependencies: Vec<Dependency>,
}

pub struct Decoder {
    decode: DecodeFn,
    match_triggers: MatchTriggersFn,

    _library: Library,
}

impl Decoder {
    pub fn load(so_path: &str) -> Result<Self, DecoderError> {
        let library = unsafe { Library::new(so_path) }.map_err(|_| DecoderError::Load)?;

        let abi_version: AbiVersionFn = unsafe { load_symbol(&library, "abi_version")? };
        let decode: DecodeFn = unsafe { load_symbol(&library, "decode")? };
        let match_triggers: MatchTriggersFn = unsafe { load_symbol(&library, "match_triggers")? };

        let got = unsafe { abi_version() };
        if got != ABI_VERSION {
            return Err(DecoderError::AbiMismatch {
                got,
                expected: ABI_VERSION,
            });
        }

        Ok(Decoder {
            decode,
            match_triggers,
            _library: library,
        })
    }

    pub fn decode(
        &self, package_path: &str, output_dir: &str, checksum: [u8; 32], cancel: &CancelToken,
    ) -> Result<DecodedPackage, DecoderError> {
        let request = CDecodeRequest {
            struct_size: size_of::<CDecodeRequest>(),
            package_path: CSlice::from_borrowed(package_path.as_bytes()),
            output_dir: CSlice::from_borrowed(output_dir.as_bytes()),
            checksum,
            cancel_token: cancel as *const CancelToken as *mut CancelToken,
        };

        let mut response = MaybeUninit::<CDecodeResponse>::uninit();

        let code = unsafe { (self.decode)(&request, response.as_mut_ptr()) };
        if code != 0 {
            return Err(DecoderError::Failed(code));
        }

        let response = unsafe { response.assume_init() };

        unsafe { response.validate() }?;

        let meta = PackageMeta::try_from(&response.meta)?;

        let dependencies = unsafe { response.dependencies.as_slice() }
            .iter()
            .map(Dependency::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DecodedPackage { meta, dependencies })
    }

    pub fn match_triggers(&self, table: &CTriggerTable) -> Result<Vec<u16>, DecoderError> {
        let capacity = unsafe { table.entries.as_slice() }.len();
        let mut ids = vec![0u16; capacity];

        let mut matches = CTriggerMatches {
            struct_size: size_of::<CTriggerMatches>(),
            ids: ids.as_mut_ptr(),
            capacity,
            len: 0,
        };

        let code = unsafe { (self.match_triggers)(table, &mut matches) };
        if code != 0 {
            return Err(DecoderError::Failed(code));
        }

        ids.truncate(matches.len.min(matches.capacity));

        Ok(ids)
    }
}
