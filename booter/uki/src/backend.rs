// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::ffi::OsStr;
use std::fs::{OpenOptions, copy, metadata};
use std::os::fd::AsRawFd;
use std::os::raw::c_long;
use std::os::unix::fs::MetadataExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use efivar::VarManager;
use efivar::boot::{
    BootEntry, BootEntryAttributes, BootVarName, EFIHardDrive, EFIHardDriveType, FilePath, FilePathList,
};
use efivar::efi::{Variable, VariableFlags};

use nix::{ioctl_read, ioctl_write_ptr};

use udev::{Device, DeviceType};

use uuid::Uuid;

use upac_types::booter::BootResourceKind;

use upac_types::traits::Booter;

use super::boot::{BOOT_NEXT_VAR, BOOT_ORDER_VAR, EFIVARFS_PATH};
use super::error::UkiError;
use super::uki::{EFI_LINUX_DIR, EFI_LINUX_REAL_PATH, FROM_SLOT, TO_SLOT};

const FS_IMMUTABLE_FL: c_long = 0x0000_0010;

ioctl_read!(fs_ioc_getflags, b'f', 1, c_long);
ioctl_write_ptr!(fs_ioc_setflags, b'f', 2, c_long);

pub struct Uki {
    manager: Box<dyn VarManager>,
}

impl Booter for Uki {
    type Error = UkiError;

    fn new() -> Result<Self, UkiError> {
        Ok(Self {
            manager: catch_unwind(AssertUnwindSafe(efivar::system))?,
        })
    }

    fn boot_resource_kind() -> BootResourceKind {
        BootResourceKind::Uki
    }

    fn set_one_shot(&mut self, entry_name: &str) -> Result<(), UkiError> {
        let id = self.find_boot_id(entry_name)?;

        let variable = Variable::new(BOOT_NEXT_VAR);
        Self::clear_immutable(&variable);

        self.manager
            .write(&variable, VariableFlags::default(), &id.to_le_bytes())?;

        Ok(())
    }

    fn confirm_boot(&mut self, entry_name: &str, esp_mount_point: &str) -> Result<(), UkiError> {
        let id = self.find_boot_id(entry_name)?;

        let mut order = self.manager.get_boot_order()?;
        order.retain(|&existing| existing != id);
        order.insert(0, id);

        Self::clear_immutable(&Variable::new(BOOT_ORDER_VAR));
        self.manager.set_boot_order(order)?;

        if entry_name == TO_SLOT {
            copy(
                Self::slot_image_path(esp_mount_point, TO_SLOT),
                Self::slot_image_path(esp_mount_point, FROM_SLOT),
            )?;
        }

        Ok(())
    }

    fn install(&mut self, esp_mount_point: &str, to_slot: &str, from_slot: &str) -> Result<(), UkiError> {
        let from_image_path = Self::slot_image_path(esp_mount_point, from_slot);
        if !from_image_path.exists() {
            copy(Self::slot_image_path(esp_mount_point, to_slot), &from_image_path)?;
        }

        let hard_drive = Self::esp_hard_drive(esp_mount_point)?;

        self.register_slot(hard_drive.clone(), to_slot)?;
        self.register_slot(hard_drive, from_slot)?;

        Ok(())
    }
}

impl Uki {
    fn slot_image_path(esp_mount_point: &str, slot: &str) -> PathBuf {
        Path::new(esp_mount_point)
            .join(EFI_LINUX_REAL_PATH)
            .join(format!("{slot}.efi"))
    }

    fn find_boot_id(&self, slot_filename: &str) -> Result<u16, UkiError> {
        let slot_file_name = format!("{}.efi", slot_filename.to_lowercase());

        for (entry, _var) in self.manager.get_boot_entries()? {
            let entry = entry?;
            let matches = entry
                .entry
                .file_path_list
                .as_ref()
                .is_some_and(|list| list.file_path.path.to_lowercase().ends_with(&slot_file_name));

            if matches {
                return Ok(entry.id);
            }
        }

        Err(UkiError::EntryNotFound)
    }

    fn esp_hard_drive(esp_mount_point: &str) -> Result<EFIHardDrive, UkiError> {
        let partition = Device::from_devnum(DeviceType::Block, metadata(esp_mount_point)?.dev())?;
        let disk = partition
            .parent_with_subsystem_devtype("block", "disk")?
            .ok_or(UkiError::WrongPartitionType)?;

        let parse = |value: Option<&OsStr>| -> Result<u64, UkiError> {
            value
                .and_then(OsStr::to_str)
                .and_then(|value| value.trim().parse().ok())
                .ok_or(UkiError::WrongPartitionType)
        };
        let sectors_per_block = parse(disk.attribute_value("queue/logical_block_size"))? / 512;

        let partition_sig = partition
            .property_value("PARTUUID")
            .and_then(OsStr::to_str)
            .and_then(|partuuid| Uuid::parse_str(partuuid).ok())
            .ok_or(UkiError::WrongPartitionType)?;

        Ok(EFIHardDrive {
            partition_number: parse(partition.attribute_value("partition"))? as u32,
            partition_start: parse(partition.attribute_value("start"))? / sectors_per_block,
            partition_size: parse(partition.attribute_value("size"))? / sectors_per_block,
            partition_sig,
            format: 0x02,
            sig_type: EFIHardDriveType::Gpt,
        })
    }

    fn register_slot(&mut self, hard_drive: EFIHardDrive, slot_filename: &str) -> Result<u16, UkiError> {
        let id = self.free_boot_id()?;

        let entry = BootEntry {
            attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
            description: slot_filename.to_owned(),
            file_path_list: Some(FilePathList {
                file_path: FilePath {
                    path: format!("{EFI_LINUX_DIR}{slot_filename}.efi"),
                },
                hard_drive,
            }),
            optional_data: Vec::new(),
        };

        self.manager.add_boot_entry(id, entry)?;

        Ok(id)
    }

    fn free_boot_id(&self) -> Result<u16, UkiError> {
        (0..u16::MAX)
            .find(|id| !self.manager.exists(&Variable::new(&id.boot_var_name())).unwrap_or(true))
            .ok_or(UkiError::NoFreeBootId)
    }

    fn clear_immutable(variable: &Variable) {
        let Ok(file) = OpenOptions::new()
            .read(true)
            .open(format!("{EFIVARFS_PATH}/{variable}"))
        else {
            return;
        };
        let fd = file.as_raw_fd();

        let mut flags: c_long = 0;
        if unsafe { fs_ioc_getflags(fd, &mut flags) }.is_err() {
            return;
        }

        if flags & FS_IMMUTABLE_FL != 0 {
            flags &= !FS_IMMUTABLE_FL;
            let _ = unsafe { fs_ioc_setflags(fd, &flags) };
        }
    }
}
