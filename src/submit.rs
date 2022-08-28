#![allow(clippy::uninit_assumed_init)]
use std::{io, os::windows::prelude::RawHandle, sync::atomic};

use crate::{
    windows::{
        NtSubmitIoRing, _NT_IORING_CREATE_REQUIRED_FLAGS_NT_IORING_CREATE_REQUIRED_FLAG_NONE,
        _NT_IORING_SQE,
    },
    Info,
};

const BS: usize = 32 * 1024;

pub struct Submitter<'a> {
    pub(crate) fd: &'a RawHandle,
    pub(crate) info: &'a Info,
    pub(crate) sq_head: *const atomic::AtomicU32,
    pub(crate) sq_tail: *const atomic::AtomicU32,
    pub(crate) sq_flags: *const atomic::AtomicI64,
}

impl<'a> Submitter<'a> {
    pub fn new(
        fd: &'a RawHandle,
        info: &'a Info,
        sq_head: *const atomic::AtomicU32,
        sq_tail: *const atomic::AtomicU32,
        sq_flags: *const atomic::AtomicI64,
    ) -> Submitter<'a> {
        Submitter {
            fd,
            info,
            sq_head,
            sq_tail,
            sq_flags,
        }
    }
    #[inline]
    fn sq_len(&self) -> usize {
        unsafe {
            let head = (*self.sq_head).load(atomic::Ordering::Acquire);
            let tail = (*self.sq_tail).load(atomic::Ordering::Acquire);

            tail.wrapping_sub(head) as usize
        }
    }
    /// Submit all queued submission queue events to the kernel.
    #[inline]
    pub fn submit(&self) -> io::Result<usize> {
        self.submit_and_wait(1, std::u32::MAX as usize)
    }

    /// Submit all queued submission queue events to the kernel and wait for at least `want`
    /// completion events to complete.
    pub fn submit_and_wait(&self, number_of_entries: u32, want: usize) -> io::Result<usize> {
        let res = unsafe {
            NtSubmitIoRing(
                *self.fd,
                _NT_IORING_CREATE_REQUIRED_FLAGS_NT_IORING_CREATE_REQUIRED_FLAG_NONE,
                number_of_entries,
                if number_of_entries == 0 {
                    &mut { std::mem::zeroed() }
                } else {
                    &mut (want as u64)
                },
            )
        };
        if res == 0 {
            Ok(self.sq_len())
        } else {
            Err(io::Error::from_raw_os_error(res as i32))
        }
    }
    /// Get the sqe ring
    pub fn get_sqe(&self) -> io::Result<_NT_IORING_SQE> {
        if !self.sq_space_left() > 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No space left in sqe ring",
            ));
        }
        let sqe;
        unsafe {
            let len = ((*self.info.0.__bindgen_anon_1.SubmissionQueue).Tail
                & self.info.0.SubmissionQueueRingMask) as usize;

            sqe = std::slice::from_raw_parts(
                (*self.info.0.__bindgen_anon_1.SubmissionQueue)
                    .Entries
                    .as_mut_ptr(),
                len + 1,
            )[len];
            (*self.info.0.__bindgen_anon_1.SubmissionQueue).Tail += 1;
        }
        Ok(sqe)
    }
    /// Get the buffer space left in the sqe ring
    pub fn sq_space_left(&self) -> usize {
        self.info.0.SubmissionQueueSize as usize - self.sq_len()
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// [`ReadFixed`](crate::opcode::ReadFixed) and [`WriteFixed`](crate::opcode::WriteFixed)
    /// operations.
    pub fn register_buffers(&self, infd: &'a RawHandle, outfd: &'a RawHandle) -> io::Result<()> {
        let _sqe = &self.get_sqe();
        let _fds = [infd, outfd];
        todo!()
    }

    /// Register files for I/O. You can use the registered files with
    /// [`Fixed`](crate::types::Fixed).
    ///
    /// Each fd may be -1, in which case it is considered "sparse", and can be filled in later with
    /// [`register_files_update`](Self::register_files_update).
    ///
    /// Note that this will wait for the ring to idle; it will only return once all active requests
    /// are complete. Use [`register_files_update`](Self::register_files_update) to avoid this.
    pub fn register_files(&self, _fds: &[RawHandle]) -> io::Result<()> {
        todo!()
    }
}
