#![allow(clippy::uninit_assumed_init)]
#[macro_use]
pub mod squeue;
pub mod cqueue;
pub mod opcode;
pub mod submit;
pub mod windows;

use cqueue::CompletionQueue;
use squeue::SubmissionQueue;
use std::io;
use submit::Submitter;
use windows::{win_ring, win_ring_queue_exit, win_ring_queue_init_ref};

pub struct IoRing<'a> {
    sq: squeue::Inner,
    cq: cqueue::Inner,
    pub info: Info<'a>,
}

/// The Info that were used to construct an [`IoRing`].
// #[derive(Clone)]
pub struct Info<'a>(pub &'a mut win_ring);

unsafe impl<'a> Send for IoRing<'a> {}
unsafe impl<'a> Sync for IoRing<'a> {}

impl<'a> IoRing<'a> {
    /// Create a new `IoRing` instance with default configuration parameters. See [`Builder`] to
    /// customize it further.
    ///
    /// The `entries` sets the size of queue,
    /// and its value should be the power of two.
    pub fn new(entries: u32) -> std::io::Result<IoRing<'a>> {
        let ring = unsafe { win_ring_queue_init_ref(entries) };
        unsafe fn setup_queue(p: *mut win_ring) -> io::Result<(squeue::Inner, cqueue::Inner)> {
            let sq = squeue::Inner::new(p);
            let cq = cqueue::Inner::new(p);

            Ok((sq, cq))
        }
        let (sq, cq) = unsafe { setup_queue(ring)? };
        Ok(IoRing {
            sq,
            cq,
            info: Info(unsafe { &mut *ring }),
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
    pub fn submit_and_wait(&self, want: u32) -> io::Result<usize> {
        self.submitter()
            .submit_and_wait(want, std::u32::MAX as usize)
    }

    #[inline]
    pub fn submitter(&self) -> Submitter<'_> {
        unsafe {
            Submitter {
                fd: &(*self.info.0).handle,
                info: &self.info,
                sq_head: std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Head as *const u32),
                sq_tail: std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Tail as *const u32),
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
                &(*self.info.0).handle,
                &self.info,
                std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Head as *const _),
                std::mem::transmute(&self.sq.sqes.as_ref().unwrap().Tail as *const _),
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
    pub unsafe fn completion_shared(&mut self) -> CompletionQueue<'_> {
        self.cq.borrow_shared()
    }
}

impl<'a> Drop for IoRing<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            win_ring_queue_exit(self.info.0);
        }
    }
}
