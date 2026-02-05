// Minimal resource limit syscalls for ch18_file0
// Only prlimit64 is needed

use axerrno::{AxError, AxResult};
use linux_raw_sys::general::{RLIM_NLIMITS, rlimit64};
use starry_core::task::get_process_data;
use starry_process::Pid;
use starry_vm::{VmMutPtr, VmPtr};

pub fn sys_prlimit64(
    pid: Pid,
    resource: u32,
    new_limit: *const rlimit64,
    old_limit: *mut rlimit64,
) -> AxResult<isize> {
    if resource >= RLIM_NLIMITS {
        return Err(AxError::InvalidInput);
    }

    let proc_data = get_process_data(pid)?;
    if let Some(old_limit) = old_limit.nullable() {
        let limit = &proc_data.rlim.read()[resource];
        old_limit.vm_write(rlimit64 {
            rlim_cur: limit.current,
            rlim_max: limit.max,
        })?;
    }

    if let Some(new_limit) = new_limit.nullable() {
        // FIXME: AnyBitPattern
        let new_limit = unsafe { new_limit.vm_read_uninit()?.assume_init() };
        if new_limit.rlim_cur > new_limit.rlim_max {
            return Err(AxError::InvalidInput);
        }

        let limit = &mut proc_data.rlim.write()[resource];
        if new_limit.rlim_max <= limit.max {
            limit.max = new_limit.rlim_max;
        } else {
            // TODO: patch resources
            // return Err(AxError::OperationNotPermitted);
            return Ok(0);
        }

        limit.current = new_limit.rlim_cur;
    }

    Ok(0)
}
