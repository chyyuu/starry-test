#![no_std]
#![feature(likely_unlikely)]
#![feature(bstr)]
#![allow(missing_docs)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[macro_use]
extern crate axlog;

extern crate alloc;

pub mod file;
pub mod mm;
pub mod signal;
pub mod syscall;
pub mod task;
pub mod time;
pub mod vfs;

/// Initialize.
pub fn init() {
    info!("Initialize VFS...");
    vfs::mount_all().expect("Failed to mount vfs");

    // Note: Timer/alarm functionality removed for minimal OS
    // info!("Initialize /proc/interrupts...");
    // info!("Initialize alarm...");
}
