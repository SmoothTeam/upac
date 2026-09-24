// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use clap::ValueEnum;
use clap::builder::PossibleValue;

use upac_abi::FsKind as FsKindAbi;
use upac_abi::{InitramfsGenerator, PartitionKind};

pub mod abi;
pub mod errors;
pub mod progress;

macro_rules! request_base {
    ($progress:expr) => {
        ::upac_types::request::RequestBase {
            on_hook: Some($crate::types::progress::on_progress),
            hook_ctx: $progress.ctx_ptr(),
            cancel_token: $crate::cancel_token_ptr(),
        }
    };
}
pub(crate) use request_base;

macro_rules! call {
    ($symbol:expr, $request:expr) => {{
        let request = $request.into();
        let result = $crate::types::abi::invoke(|error| unsafe { ($symbol)(request, error) });
        unsafe { request.free() };
        result
    }};
}
pub(crate) use call;

macro_rules! query {
    ($symbol:expr, $request:expr => $response_type:ty) => {{
        let request = $request.into();
        let result =
            $crate::types::abi::invoke_with_response(|response, error| unsafe { ($symbol)(request, response, error) });
        unsafe { request.free() };

        result.and_then(|response| {
            let converted = <$response_type>::try_from(&response)
                .map_err(|_| ::anyhow::anyhow!(::i18n_embed_fl::fl!($crate::locale::LOADER, "err-invalid-entry")));
            unsafe { response.free() };
            converted
        })
    }};
}
pub(crate) use query;

#[cfg(test)]
#[path = "../../tests/inline/types.rs"]
mod tests;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FsKind(pub FsKindAbi);

impl From<FsKind> for FsKindAbi {
    fn from(value: FsKind) -> Self {
        value.0
    }
}

impl ValueEnum for FsKind {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            FsKind(FsKindAbi::Ext4),
            FsKind(FsKindAbi::Btrfs),
            FsKind(FsKindAbi::Xfs),
            FsKind(FsKindAbi::Vfat),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self.0 {
            FsKindAbi::Ext4 => PossibleValue::new("ext4"),
            FsKindAbi::Btrfs => PossibleValue::new("btrfs"),
            FsKindAbi::Xfs => PossibleValue::new("xfs"),
            FsKindAbi::Vfat => PossibleValue::new("vfat"),
        })
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PartitionKindClapArg(pub PartitionKind);

impl From<PartitionKindClapArg> for PartitionKind {
    fn from(value: PartitionKindClapArg) -> Self {
        value.0
    }
}

impl ValueEnum for PartitionKindClapArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            PartitionKindClapArg(PartitionKind::Linux),
            PartitionKindClapArg(PartitionKind::Root),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self.0 {
            PartitionKind::Linux => Some(PossibleValue::new("linux")),
            PartitionKind::Root => Some(PossibleValue::new("root")),
            PartitionKind::Esp => None,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct InitramfsGeneratorClapArg(pub InitramfsGenerator);

impl From<InitramfsGeneratorClapArg> for InitramfsGenerator {
    fn from(value: InitramfsGeneratorClapArg) -> Self {
        value.0
    }
}

impl ValueEnum for InitramfsGeneratorClapArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            InitramfsGeneratorClapArg(InitramfsGenerator::Dracut),
            InitramfsGeneratorClapArg(InitramfsGenerator::Mkinitcpio),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self.0 {
            InitramfsGenerator::Dracut => PossibleValue::new("dracut"),
            InitramfsGenerator::Mkinitcpio => PossibleValue::new("mkinitcpio"),
        })
    }
}

#[derive(Clone, Copy)]
pub enum BootPlugin {
    Uki,
    SystemdBoot,
    Grub,
    Refind,
}

impl BootPlugin {
    pub fn as_str(&self) -> &'static str {
        match self {
            BootPlugin::Uki => "uki",
            BootPlugin::SystemdBoot => "systemd-boot",
            BootPlugin::Grub => "grub",
            BootPlugin::Refind => "refind",
        }
    }
}

impl ValueEnum for BootPlugin {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            BootPlugin::Uki,
            BootPlugin::SystemdBoot,
            BootPlugin::Grub,
            BootPlugin::Refind,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.as_str()))
    }
}

pub fn parse_size_mib(raw: &str) -> Result<u64, String> {
    let trimmed = raw.trim();

    let Some(split_at) = trimmed.find(|character: char| !character.is_ascii_digit()) else {
        return trimmed.parse().map_err(|_| format!("invalid size: \"{raw}\""));
    };

    let (number, unit) = trimmed.split_at(split_at);
    let number: u64 = number.parse().map_err(|_| format!("invalid size: \"{raw}\""))?;

    let bytes_per_unit: u64 = match unit.to_ascii_uppercase().as_str() {
        "K" | "KIB" => 1024,
        "M" | "MIB" => 1024 * 1024,
        "G" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TIB" => 1024 * 1024 * 1024 * 1024,
        "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        _ => return Err(format!("unknown size unit: \"{unit}\"")),
    };

    Ok(number * bytes_per_unit / (1024 * 1024))
}
