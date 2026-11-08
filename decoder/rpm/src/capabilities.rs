// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::decoder::DecodeError;

use super::layout::capabilities::{ALL, EFFECTIVE, NAMES, PREFIX, REVISION};

#[derive(Default)]
pub struct FileCapabilities {
    permitted: u64,
    inheritable: u64,
    effective: u64,
}

impl FileCapabilities {
    pub fn parse(text: &str) -> Result<Self, DecodeError> {
        fn shift_capability_bit(index: usize) -> Result<u64, DecodeError> {
            u32::try_from(index)
                .ok()
                .and_then(|index| 1u64.checked_shl(index))
                .ok_or(DecodeError::MalformedMetadata)
        }

        fn parse_mask(list: &str) -> Result<u64, DecodeError> {
            list.split(',').try_fold(0, |mask, name| {
                let name = name.to_ascii_lowercase();

                if name == ALL {
                    return Ok(mask | (shift_capability_bit(NAMES.len())? - 1));
                }

                let name = name.strip_prefix(PREFIX).unwrap_or(&name);
                let index = match NAMES.iter().position(|known| *known == name) {
                    Some(index) => index,
                    None => name.parse().map_err(|_| DecodeError::MalformedMetadata)?,
                };

                Ok(mask | shift_capability_bit(index)?)
            })
        }

        let mut capabilities = Self::default();

        for clause in text.split_whitespace() {
            let split = clause.find(['=', '+', '-']).ok_or(DecodeError::MalformedMetadata)?;
            let (list, actions) = clause.split_at(split);

            let mask = match list {
                "" if actions.starts_with('=') => parse_mask(ALL)?,
                "" => return Err(DecodeError::MalformedMetadata),
                list => parse_mask(list)?,
            };

            let mut raise = true;

            for symbol in actions.chars() {
                match symbol {
                    '=' => {
                        capabilities.permitted &= !mask;
                        capabilities.inheritable &= !mask;
                        capabilities.effective &= !mask;
                        raise = true;
                    }
                    '+' => raise = true,
                    '-' => raise = false,
                    flag => {
                        let set = match flag {
                            'p' => &mut capabilities.permitted,
                            'i' => &mut capabilities.inheritable,
                            'e' => &mut capabilities.effective,
                            _ => return Err(DecodeError::MalformedMetadata),
                        };

                        if raise {
                            *set |= mask;
                        } else {
                            *set &= !mask;
                        }
                    }
                }
            }
        }

        Ok(capabilities)
    }

    pub fn to_xattr(&self) -> Box<[u8]> {
        let magic = if self.effective == 0 {
            REVISION
        } else {
            REVISION | EFFECTIVE
        };

        let permitted = self.permitted.to_le_bytes();
        let inheritable = self.inheritable.to_le_bytes();

        [
            &magic.to_le_bytes()[..],
            &permitted[..4],
            &inheritable[..4],
            &permitted[4..],
            &inheritable[4..],
        ]
        .concat()
        .into_boxed_slice()
    }
}
