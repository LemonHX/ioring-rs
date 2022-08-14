use std::{io, mem, os::windows::prelude::RawHandle, sync::atomic};

use windows::Win32::Storage::FileSystem::{
    SubmitIoRing, IORING_BUFFER_INFO, IORING_CREATE_FLAGS, IORING_OP_READ,
    IORING_OP_REGISTER_BUFFERS, IORING_SQE,
};

use crate::Info;

const BS: usize = 32 * 1024;

pub struct Submitter<'a> {
    pub(crate) fd: &'a RawHandle,
    pub(crate) info: &'a Info,
    pub(crate) sq_head: *const atomic::AtomicU32,
    pub(crate) sq_tail: *const atomic::AtomicU32,
    pub(crate) sq_flags: *const atomic::AtomicU32,
}

impl<'a> Submitter<'a> {
    pub fn new(
        fd: &'a RawHandle,
        info: &'a Info,
        sq_head: *const atomic::AtomicU32,
        sq_tail: *const atomic::AtomicU32,
        sq_flags: *const atomic::AtomicU32,
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
        self.submit_and_wait(0)
    }

    /// Submit all queued submission queue events to the kernel and wait for at least `want`
    /// completion events to complete.
    pub fn submit_and_wait(&self, want: usize) -> io::Result<usize> {
        let res = unsafe { SubmitIoRing(std::mem::transmute(self.fd), 0, want as u32) }.unwrap();
        if res == 0 {
            Ok(self.sq_len())
        } else {
            Err(io::Error::from_raw_os_error(res as i32))
        }
    }
    /// Get the sqe ring
    pub fn get_sqe(&self) -> io::Result<IORING_SQE> {
        if !self.sq_space_left() > 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No space left in sqe ring",
            ));
        }

        let sqe = unsafe {
            std::slice::from_raw_parts(
                (*self.info.0.SubmissionQueue).Entries,
                mem::size_of::<IORING_SQE>(),
            )[((*self.info.0.SubmissionQueue).Tail
                & self.info.0.SubmissionQueueSizeMask) as usize]
        };
        unsafe {
            (*self.info.0.SubmissionQueue).Tail = (*self.info.0.SubmissionQueue).Tail + 1;
        }
        Ok(sqe)
    }
    /// Get the buffer space left in the sqe ring
    pub fn sq_space_left(&self) -> usize {
        return self.info.0.SubmissionQueueSize as usize - self.sq_len();
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// [`ReadFixed`](crate::opcode::ReadFixed) and [`WriteFixed`](crate::opcode::WriteFixed)
    /// operations.
    pub fn register_buffers(&self, infd: &'a RawHandle, outfd: &'a RawHandle) -> io::Result<()> {
        let sqe = &self.get_sqe();
        let fds = [infd, outfd];

        Ok(())
    }

    /// Register files for I/O. You can use the registered files with
    /// [`Fixed`](crate::types::Fixed).
    ///
    /// Each fd may be -1, in which case it is considered "sparse", and can be filled in later with
    /// [`register_files_update`](Self::register_files_update).
    ///
    /// Note that this will wait for the ring to idle; it will only return once all active requests
    /// are complete. Use [`register_files_update`](Self::register_files_update) to avoid this.
    pub fn register_files(&self, fds: &[RawHandle]) -> io::Result<()> {
        Ok(())
    }
}