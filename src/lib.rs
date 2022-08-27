#![allow(clippy::uninit_assumed_init)]
#[macro_use]
pub mod squeue;
pub mod cqueue;
pub mod opcode;
pub mod submit;
pub mod windows;

use cqueue::CompletionQueue;
use squeue::SubmissionQueue;
use std::{io, os::windows::prelude::RawHandle};
use submit::Submitter;
use windows::{_NT_IORING_INFO, _NT_IORING_STRUCTV1};

use crate::windows::{
    NtCreateIoRing, _IORING_VERSION_IORING_VERSION_3,
    _NT_IORING_CREATE_ADVISORY_FLAGS_NT_IORING_CREATE_ADVISORY_FLAG_NONE, _NT_IORING_CREATE_FLAGS,
    _NT_IORING_CREATE_REQUIRED_FLAGS_NT_IORING_CREATE_REQUIRED_FLAG_NONE,
};

pub struct IoRing {
    sq: squeue::Inner,
    cq: cqueue::Inner,
    info: Info,
    handle: RawHandle,
}

/// IoRing build info
#[derive(Clone, Default)]
pub struct Builder {
    dontfork: bool,
    info: _NT_IORING_INFO,
}

/// The Info that were used to construct an [`IoRing`].
#[derive(Clone)]
pub struct Info(_NT_IORING_INFO);

unsafe impl Send for IoRing {}
unsafe impl Sync for IoRing {}

impl IoRing {
    /// Create a new `IoRing` instance with default configuration parameters. See [`Builder`] to
    /// customize it further.
    ///
    /// The `entries` sets the size of queue,
    /// and its value should be the power of two.
    pub fn new(entries: u32) -> std::io::Result<IoRing> {
        IoRing::with_params(entries, Default::default())
    }

    /// Create a [`Builder`] for an `IoUring` instance.
    ///
    /// This allows for further customization than [`new`](Self::new).
    #[must_use]
    pub fn builder() -> Builder {
        Builder {
            dontfork: false,
            info: _NT_IORING_INFO::default(),
        }
    }
    fn with_params(entries: u32, mut p: _NT_IORING_INFO) -> std::io::Result<IoRing> {
        let mut handle: RawHandle = unsafe { std::mem::zeroed() };
        let mut ioring_struct = _NT_IORING_STRUCTV1 {
            IoRingVersion: _IORING_VERSION_IORING_VERSION_3,
            SubmissionQueueSize: entries,
            CompletionQueueSize: entries * 2,
            Flags: _NT_IORING_CREATE_FLAGS {
                Required: _NT_IORING_CREATE_REQUIRED_FLAGS_NT_IORING_CREATE_REQUIRED_FLAG_NONE,
                Advisory: _NT_IORING_CREATE_ADVISORY_FLAGS_NT_IORING_CREATE_ADVISORY_FLAG_NONE,
            },
        };
        let res = unsafe {
            NtCreateIoRing(
                &mut handle,
                std::mem::size_of::<_NT_IORING_STRUCTV1>() as u32,
                &mut ioring_struct,
                std::mem::size_of::<_NT_IORING_INFO>() as u32,
                &mut p,
            )
        };
        dbg!(res);
        #[inline]
        unsafe fn setup_queue(p: &_NT_IORING_INFO) -> io::Result<(squeue::Inner, cqueue::Inner)> {
            let sq = squeue::Inner::new(p);
            let cq = cqueue::Inner::new(p);

            Ok((sq, cq))
        }

        let (sq, cq) = unsafe { setup_queue(&p)? };
        Ok(IoRing {
            sq,
            cq,
            info: Info(p),
            handle,
        })
    }

    /// Get the Info that were used to construct this instance.
    #[inline]
    pub fn info(&self) -> &Info {
        &self.info
    }

    /// Initiate asynchronous I/O. See [`Submitter::submit`] for more details.
    #[inline]
    pub fn submit(&self) -> io::Result<usize> {
        self.submitter().submit()
    }

    /// Initiate and/or complete asynchronous I/O. See [`Submitter::submit_and_wait`] for more
    /// details.
    #[inline]
    pub fn submit_and_wait(&self, want: usize) -> io::Result<usize> {
        self.submitter().submit_and_wait(std::u32::MAX, want)
    }

    #[inline]
    pub fn submitter(&self) -> Submitter<'_> {
        unsafe {
            Submitter {
                fd: &self.handle,
                info: &self.info,
                sq_head: std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Head as *const u32),
                sq_tail: std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Tail as *const u32),
                sq_flags: std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Flags as *const i32),
            }
        }
    }
    /// Get the submitter, submission queue and completion queue of the io_uring instance. This can
    /// be used to operate on the different parts of the io_uring instance independently.
    ///
    /// If you use this method to obtain `sq` and `cq`,
    /// please note that you need to `drop` or `sync` the queue before and after submit,
    /// otherwise the queue will not be updated.
    #[inline]
    pub fn split(&mut self) -> (Submitter<'_>, SubmissionQueue<'_>, CompletionQueue<'_>) {
        unsafe {
            let submit = Submitter::new(
                &self.handle,
                &self.info,
                std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Head as *const _),
                std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Tail as *const _),
                std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Flags as *const _),
            );
            (submit, self.sq.borrow(), self.cq.borrow())
        }
    }

    /// Get the submission queue of the io_uring instance. This is used to send I/O requests to the
    /// kernel.
    #[inline]
    pub fn submission(&mut self) -> SubmissionQueue<'_> {
        self.sq.borrow()
    }

    /// Get the submission queue of the io_uring instance from a shared reference.
    ///
    /// # Safety
    ///
    /// No other [`SubmissionQueue`]s may exist when calling this function.
    #[inline]
    pub unsafe fn submission_shared(&self) -> SubmissionQueue<'_> {
        self.sq.borrow_shared()
    }

    /// Get completion queue of the io_uring instance. This is used to receive I/O completion
    /// events from the kernel.
    #[inline]
    pub fn completion(&mut self) -> CompletionQueue<'_> {
        self.cq.borrow()
    }

    /// Get the completion queue of the io_uring instance from a shared reference.
    ///
    /// # Safety
    ///
    /// No other [`CompletionQueue`]s may exist when calling this function.
    #[inline]
    pub unsafe fn completion_shared(&self) -> CompletionQueue<'_> {
        self.cq.borrow_shared()
    }
}
