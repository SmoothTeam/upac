// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{File, copy, metadata};
use std::io::ErrorKind as IoErrorKind;
use std::os::unix::fs::MetadataExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use efivar::boot::{
    BootEntry, BootEntryAttributes, BootVarName, EFIHardDrive, EFIHardDriveType, FilePath, FilePathList,
};
use efivar::efi::{Variable, VariableFlags};
use efivar::{Error as EfivarError, VarManager, system};

use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};

use udev::{Device, DeviceType};

use uuid::Uuid;

use upac_abi::BOOT_ABI_VERSION;
use upac_abi::request::booter::{
    CBootPluginConfirmSuccessBootRequest, CBootPluginInstallRequest, CBootPluginSetOneShotRequest,
};

use upac_booter_kit::{BootBackend, BuiltinBooter};

use upac_types::booter::{BootError, BootResourceKind};
use upac_types::plugin::plugin_status;
use upac_types::request::booter::{
    BootPluginConfirmSuccessBootRequest, BootPluginInstallRequest, BootPluginSetOneShotRequest,
};

use self::layout::device_path::{PARTITION_FORMAT, SECTOR_SIZE};
use self::layout::efivarfs::PATH as EFIVARFS_PATH;
use self::layout::esp::{IMAGE_EXTENSION, LINUX_DIRECTORY, LINUX_PATH};
use self::layout::manifest::NAME;
use self::layout::slots::{FROM as FROM_SLOT, TO as TO_SLOT};
use self::layout::udev::{
    DISK_DEVTYPE, LOGICAL_BLOCK_SIZE, PARTITION_NUMBER, PARTITION_SIZE, PARTITION_START, PARTITION_UUID, SUBSYSTEM,
};
use self::layout::variables::{BOOT_NEXT, BOOT_ORDER};

mod layout {
    include!(concat!(env!("OUT_DIR"), "/layout.rs"));
}

pub const BOOTER: BuiltinBooter = BuiltinBooter {
    name: NAME,
    boot_resource_kind,
    set_one_shot,
    confirm_boot,
    install,
};

#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub extern "C" fn boot_abi_version() -> u32 {
    BOOT_ABI_VERSION
}

#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub extern "C" fn boot_resource_kind() -> u8 {
    Uki::boot_resource_kind().into()
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginSetOneShotRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn set_one_shot(request: *const CBootPluginSetOneShotRequest) -> i32 {
    fn set_one_shot_entry(request: &CBootPluginSetOneShotRequest) -> Result<(), BootError> {
        let request = BootPluginSetOneShotRequest::try_from(request)?;

        Uki::new()?.set_one_shot(&request.entry_name)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(set_one_shot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginConfirmSuccessBootRequest` for
/// the duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn confirm_boot(request: *const CBootPluginConfirmSuccessBootRequest) -> i32 {
    fn confirm_boot_entry(request: &CBootPluginConfirmSuccessBootRequest) -> Result<(), BootError> {
        let request = BootPluginConfirmSuccessBootRequest::try_from(request)?;

        Uki::new()?.confirm_boot(&request.entry_name, &request.esp_mount_point)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(confirm_boot_entry(unsafe { &*request }))
}

/// # Safety
/// `request`, if non-null, must point to a valid, initialized `CBootPluginInstallRequest` for the
/// duration of the call.
#[cfg_attr(feature = "cdylib", unsafe(no_mangle))]
pub unsafe extern "C" fn install(request: *const CBootPluginInstallRequest) -> i32 {
    fn install_booter(request: &CBootPluginInstallRequest) -> Result<(), BootError> {
        let request = BootPluginInstallRequest::try_from(request)?;

        Uki::new()?.install(&request.esp_mount_point, &request.to_slot, &request.from_slot)
    }

    if request.is_null() {
        return plugin_status(Err(BootError::InvalidRequest));
    }

    plugin_status(install_booter(unsafe { &*request }))
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

    fn boot_resource_kind() -> BootResourceKind {
        BootResourceKind::Uki
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), BootError> {
        let boot_id = self.find_boot_id(entry_name)?;

        let variable = Variable::new(BOOT_NEXT);
        clear_immutable(&variable);

        self.manager
            .write(&variable, VariableFlags::default(), &boot_id.to_le_bytes())
            .map_err(convert_efivar_error)
    }

    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), BootError> {
        let boot_id = self.find_boot_id(entry_name)?;

        let mut boot_order = self.manager.get_boot_order().map_err(convert_efivar_error)?;
        boot_order.retain(|&existing_id| existing_id != boot_id);
        boot_order.insert(0, boot_id);

        clear_immutable(&Variable::new(BOOT_ORDER));
        self.manager.set_boot_order(boot_order).map_err(convert_efivar_error)?;

        if entry_name == TO_SLOT {
            copy_slot_image(esp_mount_point, TO_SLOT, FROM_SLOT)?;
        }

        Ok(())
    }

    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), BootError> {
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

        if !build_slot_image_path(esp_mount_point, from_slot).exists() {
            copy_slot_image(esp_mount_point, to_slot, from_slot)?;
        }

        let hard_drive = read_esp_hard_drive(esp_mount_point)?;

        self.register_slot(hard_drive.clone(), to_slot)?;
        self.register_slot(hard_drive, from_slot)
    }
}

impl Uki {
    fn find_boot_id(&self, slot: &str) -> Result<u16, BootError> {
        let image_name = format!("{}.{IMAGE_EXTENSION}", slot.to_lowercase());

        for (entry, _variable) in self.manager.get_boot_entries().map_err(convert_efivar_error)? {
            let entry = entry.map_err(convert_efivar_error)?;

            let matches = entry
                .entry
                .file_path_list
                .as_ref()
                .is_some_and(|list| list.file_path.path.to_lowercase().ends_with(&image_name));

            if matches {
                return Ok(entry.id);
            }
        }

        Err(BootError::EntryNotFound)
    }

    fn register_slot(&mut self, hard_drive: EFIHardDrive, slot: &str) -> Result<(), BootError> {
        let boot_id = (0..u16::MAX)
            .find(|id| !self.manager.exists(&Variable::new(&id.boot_var_name())).unwrap_or(true))
            .ok_or(BootError::NoFreeBootId)?;

        let entry = BootEntry {
            attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
            description: slot.to_owned(),
            file_path_list: Some(FilePathList {
                file_path: FilePath {
                    path: format!("{LINUX_DIRECTORY}{slot}.{IMAGE_EXTENSION}"),
                },
                hard_drive,
            }),
            optional_data: Vec::new(),
        };

        self.manager
            .add_boot_entry(boot_id, entry)
            .map_err(convert_efivar_error)
    }
}

fn build_slot_image_path(esp_mount_point: &str, slot: &str) -> PathBuf {
    Path::new(esp_mount_point)
        .join(LINUX_PATH)
        .join(format!("{slot}.{IMAGE_EXTENSION}"))
}

fn copy_slot_image(esp_mount_point: &str, source_slot: &str, target_slot: &str) -> Result<(), BootError> {
    copy(
        build_slot_image_path(esp_mount_point, source_slot),
        build_slot_image_path(esp_mount_point, target_slot),
    )
    .map(|_| ())
    .map_err(|error| match error.kind() {
        IoErrorKind::NotFound => BootError::EntryNotFound,
        _ => error.into(),
    })
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
