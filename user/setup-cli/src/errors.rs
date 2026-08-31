// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result as FmtResult};

use upac_setup::error::SetupError;

#[derive(Debug)]
pub struct LocalizedSetupError(pub SetupError);

impl Display for LocalizedSetupError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            SetupError::Common(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_common"))
            }
            SetupError::Mount(errno) => {
                write!(formatter, "{} ({errno})", gettextrs::gettext("err_mount"))
            }
            SetupError::Repo(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_repo"))
            }
            SetupError::Database(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_database"))
            }
            SetupError::DeployRecord(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_deploy_record"))
            }
            SetupError::Boot(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_boot"))
            }
            SetupError::BootPlugin(error) => {
                write!(formatter, "{} ({error:?})", gettextrs::gettext("err_boot_plugin"))
            }
            SetupError::Io(kind) => {
                write!(formatter, "{} ({kind:?})", gettextrs::gettext("err_io"))
            }
            SetupError::MetaMalformed => formatter.write_str(&gettextrs::gettext("err_meta_malformed")),
            SetupError::NoSpaceLeft => formatter.write_str(&gettextrs::gettext("err_no_space_left")),
            SetupError::NotBlockDevice => formatter.write_str(&gettextrs::gettext("err_not_block_device")),
            SetupError::MkfsFailed => formatter.write_str(&gettextrs::gettext("err_mkfs_failed")),
            SetupError::WipeFailed => formatter.write_str(&gettextrs::gettext("err_wipe_failed")),
            SetupError::PartitionNotReady => formatter.write_str(&gettextrs::gettext("err_partition_not_ready")),
            SetupError::InvalidPartitionLayout => {
                formatter.write_str(&gettextrs::gettext("err_invalid_partition_layout"))
            }
            SetupError::RereadFailed(errno) => {
                write!(formatter, "{} ({errno})", gettextrs::gettext("err_reread_failed"))
            }
            SetupError::Unexpected => formatter.write_str(&gettextrs::gettext("err_unexpected")),
        }
    }
}

impl std::error::Error for LocalizedSetupError {}
