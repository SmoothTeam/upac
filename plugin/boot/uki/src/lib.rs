// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{File, metadata};
use std::os::unix::fs::MetadataExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Component, Path, PathBuf};

use efivar::boot::{
    BootEntry, BootEntryAttributes, BootVarName, EFIHardDrive, EFIHardDriveType, FilePath, FilePathList,
};
use efivar::efi::{Variable, VariableFlags};
use efivar::{Error as EfivarError, VarManager, system};

use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};

use udev::{Device, DeviceType};

use uuid::Uuid;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::{CBootPluginEntryRequest, CBootPluginInstallRequest};
use upac_abi::response::CBootPluginInfo;

use upac_booter_kit::BootBackend;

use upac_types::booter::{BootError, BootResourceKind};
use upac_types::decoder::plugin_status;
use upac_types::request::{BootPluginEntryRequest, BootPluginInstallRequest};
use upac_types::response::BootPluginInfo;

use self::layout::device_path::{PARTITION_FORMAT, SECTOR_SIZE, SEPARATOR};
use self::layout::efivarfs::PATH as EFIVARFS_PATH;
use self::layout::udev::{
    DISK_DEVTYPE, LOGICAL_BLOCK_SIZE, PARTITION_NUMBER, PARTITION_SIZE, PARTITION_START, PARTITION_UUID, SUBSYSTEM,
};
use self::layout::variables::{BOOT_NEXT, BOOT_ORDER};

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

#[unsafe(no_mangle)]
pub extern "C" fn boot_plugin_info() -> CBootPluginInfo {
    BootPluginInfo {
        abi_version: BOOT_ABI_VERSION,
        kind: BootResourceKind::Uki,
    }
    .into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginInstallRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn install(request: *const CBootPluginInstallRequest) -> i32 {
    fn install_booter(request: &CBootPluginInstallRequest) -> Result<(), BootError> {
        let request = BootPluginInstallRequest::try_from(request)?;

        Uki::new()?.install(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(install_booter(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginEntryRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginEntryRequest) -> i32 {
    fn set_one_shot_entry(request: &CBootPluginEntryRequest) -> Result<(), BootError> {
        let request = BootPluginEntryRequest::try_from(request)?;

        Uki::new()?.set_one_shot(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_one_shot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginEntryRequest` for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_default(request: *const CBootPluginEntryRequest) -> i32 {
    fn set_default_entry(request: &CBootPluginEntryRequest) -> Result<(), BootError> {
        let request = BootPluginEntryRequest::try_from(request)?;

        Uki::new()?.set_default(&request)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_default_entry(unsafe { &*request }))
}

struct Uki {
    manager: Box<dyn VarManager>,
}

impl BootBackend for Uki {
    type Error = BootError;

    fn new() -> Result<Self, BootError> {
        let manager = catch_unwind(AssertUnwindSafe(system)).map_err(|_| BootError::EfiUnavailable)?;

        Ok(Self { manager })
    }

    fn install(&mut self, request: &BootPluginInstallRequest) -> Result<(), BootError> {
        let _ = request;

        Ok(())
    }

    fn set_one_shot(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        let boot_id = self.register_entry(request)?;

        let variable = Variable::new(BOOT_NEXT);
        clear_immutable(&variable);

        self.manager
            .write(&variable, VariableFlags::default(), &boot_id.to_le_bytes())
            .map_err(convert_efivar_error)
    }

    fn set_default(&mut self, request: &BootPluginEntryRequest) -> Result<(), BootError> {
        let boot_id = self.register_entry(request)?;

        let mut boot_order = self.manager.get_boot_order().map_err(convert_efivar_error)?;
        self.remove_stale_entries(request, &mut boot_order)?;

        boot_order.retain(|&existing_id| existing_id != boot_id);
        boot_order.insert(0, boot_id);

        clear_immutable(&Variable::new(BOOT_ORDER));
        self.manager.set_boot_order(boot_order).map_err(convert_efivar_error)
    }
}

impl Uki {
    fn register_entry(&mut self, request: &BootPluginEntryRequest) -> Result<u16, BootError> {
        fn read_esp_hard_drive(esp_mount_point: &str) -> Result<EFIHardDrive, BootError> {
            let partition = Device::from_devnum(DeviceType::Block, metadata(esp_mount_point)?.dev())?;
            let disk = partition
                .parent_with_subsystem_devtype(SUBSYSTEM, DISK_DEVTYPE)?
                .ok_or(BootError::WrongPartitionType)?;

            let parse_number = |value: Option<&OsStr>| -> Result<u64, BootError> {
                value
                    .and_then(OsStr::to_str)
                    .and_then(|value| value.trim().parse().ok())
                    .ok_or(BootError::WrongPartitionType)
            };

            let sectors_per_block = parse_number(disk.attribute_value(LOGICAL_BLOCK_SIZE))? / u64::from(SECTOR_SIZE);
            if sectors_per_block == 0 {
                return Err(BootError::WrongPartitionType);
            }

            let partition_number = u32::try_from(parse_number(partition.attribute_value(PARTITION_NUMBER))?)
                .map_err(|_| BootError::WrongPartitionType)?;

            let partition_signature = partition
                .property_value(PARTITION_UUID)
                .and_then(OsStr::to_str)
                .and_then(|partition_uuid| Uuid::parse_str(partition_uuid).ok())
                .ok_or(BootError::WrongPartitionType)?;

            let format = u8::try_from(PARTITION_FORMAT).map_err(|_| BootError::Unexpected)?;

            Ok(EFIHardDrive {
                partition_number,
                partition_start: parse_number(partition.attribute_value(PARTITION_START))? / sectors_per_block,
                partition_size: parse_number(partition.attribute_value(PARTITION_SIZE))? / sectors_per_block,
                partition_sig: partition_signature,
                format,
                sig_type: EFIHardDriveType::Gpt,
            })
        }

        let device_path = build_device_path(&request.entry_path);

        if let Some(boot_id) = self.find_boot_id(&device_path)? {
            return Ok(boot_id);
        }

        let hard_drive = read_esp_hard_drive(&request.esp_mount_point)?;

        let boot_id = (0..u16::MAX)
            .find(|id| !self.manager.exists(&Variable::new(&id.boot_var_name())).unwrap_or(true))
            .ok_or(BootError::NoFreeBootId)?;

        let entry = BootEntry {
            attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
            description: request.entry_id.clone(),
            file_path_list: Some(FilePathList {
                file_path: FilePath { path: device_path },
                hard_drive,
            }),
            optional_data: Vec::new(),
        };

        self.manager
            .add_boot_entry(boot_id, entry)
            .map_err(convert_efivar_error)?;

        Ok(boot_id)
    }

    fn find_boot_id(&self, device_path: &str) -> Result<Option<u16>, BootError> {
        for (entry, _variable) in self.manager.get_boot_entries().map_err(convert_efivar_error)? {
            let Ok(entry) = entry else {
                continue;
            };

            let matches = entry
                .entry
                .file_path_list
                .as_ref()
                .is_some_and(|list| list.file_path.path.eq_ignore_ascii_case(device_path));

            if matches {
                return Ok(Some(entry.id));
            }
        }

        Ok(None)
    }

    fn remove_stale_entries(
        &mut self, request: &BootPluginEntryRequest, boot_order: &mut Vec<u16>,
    ) -> Result<(), BootError> {
        let device_path = build_device_path(&request.entry_path);
        let Some((directory, _)) = device_path.rsplit_once(SEPARATOR) else {
            return Ok(());
        };
        let directory = format!("{directory}{SEPARATOR}").to_lowercase();

        let mut stale_ids = Vec::new();

        for (entry, _variable) in self.manager.get_boot_entries().map_err(convert_efivar_error)? {
            let Ok(entry) = entry else {
                continue;
            };

            let Some(list) = entry.entry.file_path_list else {
                continue;
            };

            if !list.file_path.path.to_lowercase().starts_with(&directory) {
                continue;
            }

            if build_esp_path(&request.esp_mount_point, &list.file_path.path).exists() {
                continue;
            }

            stale_ids.push(entry.id);
        }

        for stale_id in stale_ids {
            let variable = Variable::new(&stale_id.boot_var_name());
            clear_immutable(&variable);

            self.manager.delete(&variable).map_err(convert_efivar_error)?;
            boot_order.retain(|&existing_id| existing_id != stale_id);
        }

        Ok(())
    }
}

fn build_device_path(entry_path: &str) -> String {
    Path::new(entry_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => name.to_str(),
            _ => None,
        })
        .fold(String::new(), |mut device_path, name| {
            device_path.push_str(SEPARATOR);
            device_path.push_str(name);
            device_path
        })
}

fn build_esp_path(esp_mount_point: &str, device_path: &str) -> PathBuf {
    device_path
        .split(SEPARATOR)
        .filter(|name| !name.is_empty())
        .fold(PathBuf::from(esp_mount_point), |esp_path, name| esp_path.join(name))
}

fn clear_immutable(variable: &Variable) {
    let Ok(file) = File::open(format!("{EFIVARFS_PATH}/{variable}")) else {
        return;
    };

    let Ok(flags) = ioctl_getflags(&file) else {
        return;
    };

    if flags.contains(IFlags::IMMUTABLE) {
        let _ = ioctl_setflags(&file, flags.difference(IFlags::IMMUTABLE));
    }
}

fn convert_efivar_error(error: EfivarError) -> BootError {
    match error {
        EfivarError::PermissionDenied { .. } => BootError::PermissionDenied,
        _ => BootError::Unexpected,
    }
}
