// Minimal task/thread syscalls for ch18_file0
// Only set_tid_address is needed

use axerrno::AxResult;
use axtask::current;
use starry_core::task::AsThread;

/// To set the clear_child_tid field in the task extended data.
///
/// The set_tid_address() always succeeds
pub fn sys_set_tid_address(clear_child_tid: usize) -> AxResult<isize> {
    let curr = current();
    curr.as_thread().set_clear_child_tid(clear_child_tid);
    Ok(curr.id().as_u64() as isize)
}
