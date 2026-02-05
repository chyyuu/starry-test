#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate axlog;

extern crate alloc;
extern crate axruntime;

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use axfs::FS_CONTEXT;

mod entry;

pub const DEFAULT_CMDLINE: &[&str] = &["/bin/sh", "-c", include_str!("init.sh")];

fn init_cmdline() -> Vec<String> {
    match option_env!("INIT_CMDLINE") {
        Some(cmdline) if !cmdline.trim().is_empty() => cmdline
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        _ => DEFAULT_CMDLINE
            .iter()
            .copied()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
    }
}

#[unsafe(no_mangle)]
fn main() {
    starry_api::init();

    let args = init_cmdline();
    let envs = [];
    let exit_code = entry::run_initproc(&args, &envs);
    info!("Init process exited with code: {exit_code:?}");

    let cx = FS_CONTEXT.lock();
    cx.root_dir()
        .unmount_all()
        .expect("Failed to unmount all filesystems");
    cx.root_dir()
        .filesystem()
        .flush()
        .expect("Failed to flush rootfs");
}

#[cfg(feature = "vf2")]
extern crate axplat_riscv64_visionfive2;
