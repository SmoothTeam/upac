use anyhow::Result;

use colored::Colorize;

use crate::cancel_token_ptr;
use crate::ffi::request::{CUnmutatedRequest, CUnmutatedResponse};
use crate::types::errors::LibError;
use crate::types::CommandContext;

#[derive(clap::Args)]
pub struct Args {}

pub fn run(_args: Args, ctx: CommandContext) -> Result<()> {
    let mut response = CUnmutatedResponse::empty();

    let request = CUnmutatedRequest::for_list_commits(
        &ctx.config.paths.repo_path,
        &ctx.config.paths.root_path,
        &ctx.config.ostree.branch,
        cancel_token_ptr(),
    );

    let return_code = unsafe { (ctx.lib.commit.list)(request, &mut response) };
    LibError::check(return_code)?;

    let commits = unsafe { response.commits.as_slice() };
    for (index, commit) in commits.iter().enumerate() {
        let checksum = unsafe { commit.checksum.as_str() };
        let subject = unsafe { commit.subject.as_str() };

        println!("{}", subject.bold());
        println!("{}", checksum.yellow());

        if index < commits.len() - 1 {
            println!();
        }
    }

    unsafe { (ctx.lib.free_response)(&mut response) };

    Ok(())
}
