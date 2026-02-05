// Minimal mm syscalls for ch18_file0
mod brk;
mod mmap;

pub use self::{brk::*, mmap::*};
