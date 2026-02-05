// Minimal file descriptor operations for ch18_file0
// Only openat, close, and fcntl are needed

use alloc::{format, string::ToString, sync::Arc};
use core::{
    ffi::{c_char, c_int},
};

use axerrno::{AxError, AxResult};
use axfs::{FS_CONTEXT, FileBackend, OpenOptions, OpenResult};
use axfs_ng_vfs::{DirEntry, FileNode, Location, NodePermission, NodeType, Reference};
use axtask::current;
use linux_raw_sys::general::*;
use starry_core::{task::AsThread, vfs::Device};

use crate::{
    file::{
        Directory, FD_TABLE, File, FileLike, Pipe, add_file_like, close_file_like, get_file_like,
        with_fs,
    },
    mm::{UserPtr, vm_load_string},
    syscall::sys::{sys_getegid, sys_geteuid},
};

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, mode: __kernel_mode_t, (uid, gid): (u32, u32)) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    options.mode(mode).user(uid, gid);
    match flags & 0b11 {
        O_RDONLY => options.read(true),
        O_WRONLY => options.write(true),
        _ => options.read(true).write(true),
    };
    if flags & O_APPEND != 0 {
        options.append(true);
    }
    if flags & O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & O_CREAT != 0 {
        options.create(true);
    }
    if flags & O_PATH != 0 {
        options.path(true);
    }
    if flags & O_EXCL != 0 {
        options.create_new(true);
    }
    if flags & O_DIRECTORY != 0 {
        options.directory(true);
    }
    if flags & O_NOFOLLOW != 0 {
        options.no_follow(true);
    }
    if flags & O_DIRECT != 0 {
        options.direct(true);
    }
    options
}

fn add_to_fd(result: OpenResult, flags: u32) -> AxResult<i32> {
    let f: Arc<dyn FileLike> = match result {
        OpenResult::File(file) => {
            // TTY devices removed for minimal OS
            Arc::new(File::new(file))
        }
        OpenResult::Dir(dir) => Arc::new(Directory::new(dir)),
    };
    if flags & O_NONBLOCK != 0 {
        f.set_nonblocking(true)?;
    }
    add_file_like(f, flags & O_CLOEXEC != 0)
}

/// Open or create a file.
pub fn sys_openat(
    dirfd: c_int,
    path: *const c_char,
    flags: i32,
    mode: __kernel_mode_t,
) -> AxResult<isize> {
    let path = vm_load_string(path)?;
    debug!("sys_openat <= {dirfd} {path:?} {flags:#o} {mode:#o}");

    let mode = mode & !current().as_thread().proc_data.umask();

    let options = flags_to_options(flags, mode, (sys_geteuid()? as _, sys_getegid()? as _));
    with_fs(dirfd, |fs| options.open(fs, path))
        .and_then(|it| add_to_fd(it, flags as _))
        .map(|fd| fd as isize)
}

pub fn sys_close(fd: c_int) -> AxResult<isize> {
    debug!("sys_close <= {fd}");
    close_file_like(fd)?;
    Ok(0)
}

fn dup_fd(old_fd: c_int, cloexec: bool) -> AxResult<isize> {
    let f = get_file_like(old_fd)?;
    let new_fd = add_file_like(f, cloexec)?;
    Ok(new_fd as _)
}

pub fn sys_fcntl(fd: c_int, cmd: c_int, arg: usize) -> AxResult<isize> {
    debug!("sys_fcntl <= fd: {fd} cmd: {cmd} arg: {arg}");

    match cmd as u32 {
        F_DUPFD => dup_fd(fd, false),
        F_DUPFD_CLOEXEC => dup_fd(fd, true),
        F_SETLK | F_SETLKW => Ok(0),
        F_OFD_SETLK | F_OFD_SETLKW => Ok(0),
        F_GETLK | F_OFD_GETLK => {
            let arg = UserPtr::<flock64>::from(arg);
            arg.get_as_mut()?.l_type = F_UNLCK as _;
            Ok(0)
        }
        F_SETFL => {
            get_file_like(fd)?.set_nonblocking(arg & (O_NONBLOCK as usize) > 0)?;
            Ok(0)
        }
        F_GETFL => {
            let f = get_file_like(fd)?;

            let mut ret = 0;
            if f.nonblocking() {
                ret |= O_NONBLOCK;
            }

            let perm = NodePermission::from_bits_truncate(f.stat()?.mode as _);
            if perm.contains(NodePermission::OWNER_WRITE) {
                if perm.contains(NodePermission::OWNER_READ) {
                    ret |= O_RDWR;
                } else {
                    ret |= O_WRONLY;
                }
            }

            Ok(ret as _)
        }
        F_GETFD => {
            let cloexec = FD_TABLE
                .read()
                .get(fd as _)
                .ok_or(AxError::BadFileDescriptor)?
                .cloexec;
            Ok(if cloexec { FD_CLOEXEC as _ } else { 0 })
        }
        F_SETFD => {
            let cloexec = arg & FD_CLOEXEC as usize != 0;
            FD_TABLE
                .write()
                .get_mut(fd as _)
                .ok_or(AxError::BadFileDescriptor)?
                .cloexec = cloexec;
            Ok(0)
        }
        F_GETPIPE_SZ => {
            let pipe = Pipe::from_fd(fd)?;
            Ok(pipe.capacity() as _)
        }
        F_SETPIPE_SZ => {
            let pipe = Pipe::from_fd(fd)?;
            pipe.resize(arg)?;
            Ok(0)
        }
        _ => {
            warn!("unsupported fcntl parameters: cmd: {cmd}");
            Ok(0)
        }
    }
}
