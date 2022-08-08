use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::os::windows::prelude::RawHandle;
use std::sync::atomic;

use windows::Win32::Storage::FileSystem::{IORING_INFO, IORING_SQE};
// use crate::sys;
use xmmap::Mmap;

use bitflags::bitflags;

pub(crate) struct Inner {
    pub(crate) head: *const atomic::AtomicU32,
    pub(crate) tail: *const atomic::AtomicU32,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) flags: *const atomic::AtomicU32,
    dropped: *const atomic::AtomicU32,

    pub(crate) sqes: *mut windows::Win32::Storage::FileSystem::IORING_SQE,
}

pub struct SubmissionQueue<'a> {
    head: u32,
    tail: u32,
    queue: &'a Inner,
}

/// An entry in the submission queue, representing a request for an I/O operation.
///
/// These can be created via the opcodes in [`opcode`](crate::opcode).
#[repr(transparent)]
#[derive(Clone)]
pub struct Entry(pub(crate) IORING_SQE);

impl Inner {
    pub(crate) unsafe fn new(p: &IORING_INFO) -> io::Result<Inner> {
        todo!()
    }
}

/// An error pushing to the submission queue due to it being full.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PushError;

impl Display for PushError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("submission queue is full")
    }
}

impl Error for PushError {}

// impl Debug for Entry {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Entry")
//             .field("op_code", &self.0.opcode)
//             .field("flags", &self.0.flags)
//             .field("user_data", &self.0.user_data)
//             .finish()
//     }
// }
