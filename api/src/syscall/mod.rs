// Minimal syscall support for ch18_file0 (18 syscalls only)
mod fs;
mod mm;
mod resources;
mod sync;
mod sys;
mod task;

use axerrno::{AxError, LinuxError};
use axhal::uspace::UserContext;
use syscalls::Sysno;

use self::{fs::*, mm::*, resources::*, sync::*, sys::*, task::*};

pub fn handle_syscall(uctx: &mut UserContext) {
    let Some(sysno) = Sysno::new(uctx.sysno()) else {
        warn!("Invalid syscall number: {}", uctx.sysno());
        uctx.set_retval(-LinuxError::ENOSYS.code() as _);
        return;
    };

    trace!("Syscall {sysno:?}");

    let result = match sysno {
        // File I/O operations (9 syscalls)
        Sysno::openat => sys_openat(
            uctx.arg0() as _,
            uctx.arg1() as _,
            uctx.arg2() as _,
            uctx.arg3() as _,
        ),
        Sysno::close => sys_close(uctx.arg0() as _),
        Sysno::read => sys_read(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),
        Sysno::write => sys_write(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),
        Sysno::lseek => sys_lseek(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),
        Sysno::fstat => sys_fstat(uctx.arg0() as _, uctx.arg1() as _),
        Sysno::fcntl => sys_fcntl(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),
        Sysno::ioctl => sys_ioctl(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),
        Sysno::readlinkat => sys_readlinkat(
            uctx.arg0() as _,
            uctx.arg1() as _,
            uctx.arg2() as _,
            uctx.arg3() as _,
        ),

        // Memory management (3 syscalls)
        Sysno::brk => sys_brk(uctx.arg0() as _),
        Sysno::mmap => sys_mmap(
            uctx.arg0(),
            uctx.arg1() as _,
            uctx.arg2() as _,
            uctx.arg3() as _,
            uctx.arg4() as _,
            uctx.arg5() as _,
        ),
        Sysno::mprotect => sys_mprotect(uctx.arg0(), uctx.arg1() as _, uctx.arg2() as _),

        // Process management (4 syscalls)
        Sysno::exit => sys_exit(uctx.arg0() as _),
        Sysno::exit_group => sys_exit_group(uctx.arg0() as _),
        Sysno::set_tid_address => sys_set_tid_address(uctx.arg0()),
        Sysno::set_robust_list => sys_set_robust_list(uctx.arg0() as _, uctx.arg1() as _),

        // Resource limits and system (2 syscalls)
        Sysno::prlimit64 => sys_prlimit64(
            uctx.arg0() as _,
            uctx.arg1() as _,
            uctx.arg2() as _,
            uctx.arg3() as _,
        ),
        Sysno::getrandom => sys_getrandom(uctx.arg0() as _, uctx.arg1() as _, uctx.arg2() as _),

        _ => {
            warn!("Unsupported syscall for ch18_file0: {sysno}");
            Err(AxError::Unsupported)
        }
    };
    debug!("Syscall {sysno} return {result:?}");

    uctx.set_retval(result.unwrap_or_else(|err| -LinuxError::from(err).code() as _) as _);
}
