// Minimal file stat syscalls for ch18_file0
// Only fstat and fstatat are needed

use core::ffi::c_char;

use axerrno::AxResult;
use linux_raw_sys::general::{AT_EMPTY_PATH, stat};
use starry_vm::{VmMutPtr, VmPtr};

use crate::{file::resolve_at, mm::vm_load_string};

/// Get file metadata by `fd` and write into `statbuf`.
pub fn sys_fstat(fd: i32, statbuf: *mut stat) -> AxResult<isize> {
    sys_fstatat(fd, core::ptr::null(), statbuf, AT_EMPTY_PATH)
}

pub fn sys_fstatat(
    dirfd: i32,
    path: *const c_char,
    statbuf: *mut stat,
    flags: u32,
) -> AxResult<isize> {
    let path = path.nullable().map(vm_load_string).transpose()?;

    debug!("sys_fstatat <= dirfd: {dirfd}, path: {path:?}, flags: {flags}");

    let loc = resolve_at(dirfd, path.as_deref(), flags)?;
    statbuf.vm_write(loc.stat()?.into())?;

    Ok(0)
}
