#![allow(clippy::uninit_assumed_init)]
use std::{io, iter, os::windows::prelude::RawHandle, sync::atomic};

use crate::cqueue::CompletionQueue;
use crate::opcode;
use crate::windows::{win_ring_get_sqe, win_ring_sqe};
use crate::{
    windows::{
        win_ring_sq_space_left, NtSubmitIoRing, IORING_BUFFER_INFO,
        _NT_IORING_CREATE_REQUIRED_FLAGS_NT_IORING_CREATE_REQUIRED_FLAG_NONE,
        _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE, _NT_IORING_REG_BUFFERS_FLAGS,
        _NT_IORING_REG_FILES_FLAGS,
    },
    Info,
};

const BS: usize = 32 * 1024;

pub struct Submitter<'a> {
    pub(crate) fd: &'a RawHandle,
    pub(crate) info: &'a Info,
    pub(crate) sq_head: *const atomic::AtomicU32,
    pub(crate) sq_tail: *const atomic::AtomicU32,
}

impl<'a> Submitter<'a> {
    pub fn new(
        fd: &'a RawHandle,
        info: &'a Info,
        sq_head: *const atomic::AtomicU32,
        sq_tail: *const atomic::AtomicU32,
    ) -> Submitter<'a> {
        Submitter {
            fd,
            info,
            sq_head,
            sq_tail,
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
    pub fn get_sqe(&self) -> io::Result<*mut win_ring_sqe> {
        if !self.sq_space_left() > 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No space left in sqe ring",
            ));
        }
        let sqe = unsafe { win_ring_get_sqe(self.info.0) };
        Ok(sqe)
    }
    /// Get the buffer space left in the sqe ring
    pub fn sq_space_left(&self) -> u32 {
        unsafe { win_ring_sq_space_left(self.info.0) }
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// [`ReadFixed`](crate::opcode::ReadFixed) and [`WriteFixed`](crate::opcode::WriteFixed)
    /// operations.
    /// This function is replica of register_files_bufs()
    pub fn register_files_bufs(&self, infd: RawHandle, outfd: RawHandle) -> io::Result<()> {
        let _fds = vec![infd, outfd];
        let _ = opcode::RegisterFiles::new(
            self.info.0,
            _fds.as_ptr() as _,
            2,
            _NT_IORING_REG_FILES_FLAGS {
                Required: 0,
                Advisory: 0,
            },
            _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
        );

        static STATIC_BUFFER: [u8; BS] = [0u8; BS];

        let buf_info = IORING_BUFFER_INFO {
            Address: &STATIC_BUFFER as *const _ as _,
            Length: BS as u32,
        };

        let _ = opcode::RegisterBuffers::new(
            self.info.0,
            &buf_info,
            1,
            _NT_IORING_REG_BUFFERS_FLAGS {
                Required: 0,
                Advisory: 0,
            },
            _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
        );
        unsafe {
            CompletionQueue::<'_>::clear_cqes(
                self.info.0 as *const _ as *mut _,
                "submit register_buffer",
            )?;
        }
        Ok(())
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// [`ReadFixed`](crate::opcode::ReadFixed) and [`WriteFixed`](crate::opcode::WriteFixed)
    /// operations.
    /// This function is replica of register_files_bufs()
    pub fn register_read_write(&self, infd: RawHandle, outfd: RawHandle) -> io::Result<()> {
        let _fds = vec![infd, outfd];
        let _ = opcode::RegisterFiles::new(
            self.info.0,
            _fds.as_ptr() as _,
            2,
            _NT_IORING_REG_FILES_FLAGS {
                Required: 0,
                Advisory: 0,
            },
            _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
        );

        static STATIC_BUFFER: [u8; BS] = [0u8; BS];

        let buf_info = IORING_BUFFER_INFO {
            Address: &STATIC_BUFFER as *const _ as _,
            Length: BS as u32,
        };

        let _ = opcode::RegisterBuffers::new(
            self.info.0,
            &buf_info,
            1,
            _NT_IORING_REG_BUFFERS_FLAGS {
                Required: 0,
                Advisory: 0,
            },
            _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
        );
        unsafe {
            CompletionQueue::<'_>::clear_cqes(
                self.info.0 as *const _ as *mut _,
                "submit register_buffer",
            )?;
        }
        todo!()
    }

    /// Register in-memory user buffers for I/O with the kernel. You can use these buffers with the
    /// This function is replica of queue_read_write_pair()
    pub fn queue_read_write_pair(&self, offset: u64, size: usize) -> io::Result<()> {
        let sqe = self.get_sqe()?;
        todo!()
    }

    /// Copy the file of series of infd to outfd
    /// This function is replica of copy_file()
    pub fn copy_file(&self, size: u64) -> io::Result<()> {
        for offset in (0..size - (BS as u64)).step_by(BS) {
            unsafe {
                self.queue_read_write_pair(offset, BS)?;
                if self.sq_space_left() < 2 {
                    CompletionQueue::clear_cqes(self.info.0, "read_write")?;
                }
            }
        }
        let offset: u64 = size - size % BS as u64;
        if offset != size {
            self.queue_read_write_pair(offset, (size - offset) as usize)?;
            unsafe {
                CompletionQueue::clear_cqes(self.info.0, "read_write")?;
            }
        }
        Ok(())
    }
}
