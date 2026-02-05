// Minimal futex syscalls for ch18_file0
// Only set_robust_list is needed

use axerrno::{AxError, AxResult};
use axtask::current;
use linux_raw_sys::general::robust_list_head;
use starry_core::task::AsThread;

pub fn sys_set_robust_list(head: *const robust_list_head, size: usize) -> AxResult<isize> {
    if size != size_of::<robust_list_head>() {
        return Err(AxError::InvalidInput);
    }
    current().as_thread().set_robust_list_head(head.addr());

    Ok(0)
}
