//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use std::mem;

use windows::Win32::Storage::FileSystem::IORING_SQE;

/// inline zeroed io improve codegen
#[inline(always)]
fn sqe_zeroed() -> IORING_SQE {
    unsafe { mem::zeroed() }
}
