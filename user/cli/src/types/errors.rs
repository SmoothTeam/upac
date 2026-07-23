use anyhow::Result;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AbiMismatch {
    pub got: u32,
    pub expected: u32,
}

impl Display for AbiMismatch {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} ({} → {})",
            gettextrs::gettext("abi_version_mismatch"),
            self.got,
            self.expected
        )
    }
}

#[derive(Debug)]
pub enum PrepareError {
    Failed { code: i32 },
    NullMeta,
}

impl std::error::Error for AbiMismatch {}

impl Display for PrepareError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed { code } => {
                write!(
                    formatter,
                    "{}: {code}",
                    gettextrs::gettext("prepare_failed")
                )
            }
            Self::NullMeta => formatter.write_str(&gettextrs::gettext("prepare_null_meta")),
        }
    }
}

#[derive(Debug)]
pub struct LibError {
    pub code: i32,
}

impl LibError {
    pub fn check(code: i32) -> Result<(), Self> {
        if code == 0 {
            Ok(())
        } else {
            Err(Self { code })
        }
    }
}

impl std::error::Error for PrepareError {}

impl Display for LibError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let key = match self.code {
            1 => "err_unexpected",
            2 => "err_oom",
            3 => "err_not_found",
            4 => "err_permission_denied",
            5 => "err_invalid_path",
            6 => "err_no_space",
            7 => "err_abi_mismatch",
            9 => "err_thread",
            10 => "err_locked",
            11 => "err_alloc",
            12 => "err_cancelled",
            13 | 55 => "err_max_retries",
            14 => "err_read",
            15 => "err_write",
            16 => "err_diff",
            17 => "err_list",
            30 => "err_missing_field",
            31 => "err_missing_section",
            32 => "err_invalid_entry",
            33 => "err_parse",
            34 => "err_write_db",
            35 => "err_malformed_meta",
            36 => "err_malformed_files",
            37 => "err_malformed_idx",
            50 => "err_already_installed",
            51 => "err_temp_not_found",
            52 => "err_checksum",
            53 => "err_checkout",
            54 => "err_install_cancelled",
            56 => "err_check_space",
            57 => "err_make",
            58 => "err_write_config",
            70 => "err_pkg_not_found",
            71 => "err_uninstall",
            72 => "err_file_map",
            73 => "err_staging",
            90 => "err_open_repo",
            91 => "err_transaction",
            92 => "err_commit",
            93 => "err_rollback",
            94 => "err_no_prev_commit",
            95 => "err_staging_checkout",
            96 => "err_atomic_swap",
            97 => "err_commit_not_found",
            98 => "err_cleanup",
            99 => "err_repo_write",
            100 => "err_mtree",
            110 => "err_already_init",
            111 => "err_create_dir",
            112 => "err_not_dir",
            113 => "err_init",
            114 => "err_dir_not_empty",
            115 => "err_init_prefix",
            116 => "err_init_prefix_extra",
            120 => "err_file_checksum",
            121 => "err_file_exists",
            _ => "err_unknown",
        };
        write!(formatter, "{} ({})", gettextrs::gettext(key), self.code)
    }
}

impl std::error::Error for LibError {}
