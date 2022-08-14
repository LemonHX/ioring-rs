use std::{io, os::windows::prelude::RawHandle, sync::atomic};

use windows::Win32::Storage::FileSystem::{
    SubmitIoRing, IORING_CREATE_FLAGS, IORING_OP_READ, IORING_OP_REGISTER_BUFFERS, IORING_SQE, IORING_BUFFER_INFO,
};

use crate::Info;

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
    pub fn get_sqe(&self) -> IORING_SQE {
        todo!()
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// [`ReadFixed`](crate::opcode::ReadFixed) and [`WriteFixed`](crate::opcode::WriteFixed)
    /// operations.
    pub fn register_buffers(&self, bufs: &[IORING_BUFFER_INFO]) -> io::Result<()> {
        let sqe = &self.get_sqe();
        sqe = &IORING_SQE {
            OpCode: todo!(),
            Flags: todo!(),
            UserData: todo!(),
            CommonOpFlags: todo!(),
            Padding: todo!(),
            File: todo!(),
            Buffer: todo!(),
            Offset: todo!(),
            Length: todo!(),
            Key: todo!(),
        };
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
