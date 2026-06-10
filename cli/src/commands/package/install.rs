use anyhow::Result;

use std::ffi::{c_void, CString};
use std::fs::File;
use std::io::Read;
use std::ptr::null_mut;

use sha2::{Digest, Sha256};

use indicatif::ProgressBar;

use crate::cancel_token_ptr;
use crate::corelib::backend::Backend;
use crate::corelib::registry::BackendRegistry;
use crate::ffi::ctypes::CSlice;
use crate::ffi::packages::{CPackage, CPackageMeta};
use crate::ffi::request::{CMutatedRequest, CPrepareRequest};
use crate::types::errors::LibError;
use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,
    #[arg(long)]
    pub backend: Option<String>,
    #[arg(long, num_args = 0..)]
    pub checksums: Vec<String>,
}

struct PreparedPackage {
    backend: Backend,
    meta_ptr: *mut CPackageMeta,
    temp_path: CSlice,
}

impl PreparedPackage {
    fn as_c_package(&self) -> CPackage {
        CPackage::new(self.meta_ptr, self.temp_path)
    }

    unsafe fn cleanup(self) {
        unsafe { (self.backend.free_meta)(self.meta_ptr) };
        unsafe { (self.backend.cleanup)(self.temp_path) };
    }
}

pub fn run(args: Args, ctx: CommandContext) -> Result<()> {
    let registry = BackendRegistry::scan(&ctx.config.paths.backends_dir)?;

    let mut prepared: Vec<PreparedPackage> = Vec::with_capacity(args.files.len());

    let result = (|| -> Result<()> {
        for (index, file_path) in args.files.iter().enumerate() {
            let backend_config = if let Some(ref backend_flag) = args.backend {
                registry.by_flag(backend_flag)
            } else {
                registry.by_extension(file_path)
            }
            .ok_or_else(|| {
                anyhow::anyhow!("{}: {file_path}", gettextrs::gettext("err_no_backend"))
            })?;

            let backend = registry.load(backend_config)?;

            let checksum_string = match args.checksums.get(index) {
                Some(have) => have.clone(),
                None => sha256_of_file(file_path)?,
            };
            let checksum = CString::new(checksum_string)?;
            let package_path = CString::new(file_path.as_str())?;

            let progress_bar = ProgressBar::new_spinner();

            let prepare_request = CPrepareRequest::new(
                &package_path,
                &ctx.tmp_path,
                &checksum,
                Some(Backend::on_hook),
                &progress_bar as *const ProgressBar as *mut c_void,
                cancel_token_ptr(),
            );

            let (meta_ptr, temp_path) = backend.prepare(&prepare_request)?;
            progress_bar.finish_and_clear();

            prepared.push(PreparedPackage {
                backend,
                meta_ptr,
                temp_path,
            });
        }

        let packages_c: Vec<CPackage> =
            prepared.iter().map(PreparedPackage::as_c_package).collect();

        let request = CMutatedRequest::for_install(
            &packages_c,
            &ctx.config.paths.repo_path,
            &ctx.config.paths.root_path,
            &ctx.tmp_path,
            &ctx.config.ostree.branch,
            None,
            null_mut(),
            cancel_token_ptr(),
        );

        let return_code = unsafe { (ctx.lib.pkg.install)(request) };
        Ok(LibError::check(return_code)?)
    })();

    for package in prepared {
        unsafe { package.cleanup() };
    }

    result
}

fn sha256_of_file(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}
