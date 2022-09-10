use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use crate::windows::{
    win_ring, win_ring_sqe_set_data64, win_ring_sqe_set_flags, _NT_IORING_SQE,
    _NT_IORING_SQE_FLAGS, _NT_IORING_SUBMISSION_QUEUE,
};

pub struct SubmissionQueue<'a> {
    head: u32,
    tail: u32,
    queue: &'a mut Inner,
}
unsafe impl Send for SubmissionQueue<'_> {}
unsafe impl Sync for SubmissionQueue<'_> {}

/// An entry in the submission queue, representing a request for an I/O operation.
///
/// These can be created via the opcodes in [`opcode`](crate::opcode).
#[repr(transparent)]
#[derive(Clone)]
pub struct Entry(pub(crate) *mut _NT_IORING_SQE);

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
pub struct Inner {
    ring_mask: u32,
    info: *mut win_ring,
    pub sqes: *mut _NT_IORING_SUBMISSION_QUEUE,
}
impl Inner {
    pub unsafe fn new(p: *mut win_ring) -> Self {
        let ring_mask = (*p).info.SubmissionQueueRingMask;
        let sqes = (*p).info.__bindgen_anon_1.SubmissionQueue;
        Self {
            ring_mask,
            sqes,
            info: p,
        }
    }

    #[inline]
    pub unsafe fn borrow_shared(&self) -> SubmissionQueue<'_> {
        SubmissionQueue {
            head: self.sqes.as_ref().unwrap().Head,
            tail: self.sqes.as_ref().unwrap().Tail,
            queue: (self as *const Self as *mut Self).as_mut().unwrap(),
        }
    }

    #[inline]
    pub fn borrow(&mut self) -> SubmissionQueue<'_> {
        unsafe { self.borrow_shared() }
    }
}

impl SubmissionQueue<'_> {
    /// Synchronize this type with the real submission queue.
    ///
    /// This will flush any entries added by [`push`](Self::push) or
    /// [`push_multiple`](Self::push_multiple) and will update the queue's length if the kernel has
    /// consumed some entries in the meantime.
    #[inline]
    pub fn sync(&mut self) {
        unsafe {
            self.tail = self.queue.sqes.as_ref().unwrap().Head;
            self.head = self.queue.sqes.as_ref().unwrap().Tail;
        }
    }

    /// Get the total number of entries in the submission queue ring buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { (*self.queue.info).info.SubmissionQueueSize as usize }
    }

    /// Get the number of submission queue events in the ring buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.tail.wrapping_sub(self.head) as usize
    }

    /// Returns `true` if the submission queue ring buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the submission queue ring buffer has reached capacity, and no more events
    /// can be added before the kernel consumes some.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Attempts to push an [`Entry`] into the queue.
    /// If the queue is full, an error is returned.
    ///
    /// # Safety
    ///
    /// Developers must ensure that parameters of the [`Entry`] (such as buffer) are valid and will
    /// be valid for the entire duration of the operation, otherwise it may cause memory problems.
    #[inline]
    pub unsafe fn push(&mut self, Entry(_entry): &Entry) -> Result<(), PushError> {
        self.sync();
        if !self.is_full() {
            // let sqe = win_ring_get_sqe(self.queue.info);
            // *sqe = **entry;
            self.sync();
            Ok(())
        } else {
            Err(PushError)
        }
    }
}

impl Entry {
    /// Set the submission event's [flags](Flags).
    #[inline]
    pub fn flags(self, flags: _NT_IORING_SQE_FLAGS) -> Entry {
        unsafe {
            win_ring_sqe_set_flags(self.0, flags);
        }
        self
    }

    /// Set the user data. This is an application-supplied value that will be passed straight
    /// through into the [completion queue entry](crate::cqueue::Entry::user_data).
    #[inline]
    pub fn user_data(self, user_data: u64) -> Entry {
        unsafe {
            win_ring_sqe_set_data64(self.0, user_data);
        }
        self
    }

    /// Set the personality of this event. You can obtain a personality using
    /// [`Submitter::register_personality`](crate::Submitter::register_personality).
    ///
    /// Requires the `unstable` feature.
    #[cfg(feature = "unstable")]
    pub fn personality(mut self, personality: u16) -> Entry {
        self.0.personality = personality;
        self
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("Entry")
                .field("op_code", &(*self.0).OpCode)
                .field("flags", &(*self.0).Flags)
                .field("user_data", &(*self.0).UserData)
                .finish()
        }
    }
}

impl Debug for SubmissionQueue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_list();
        let mut pos = self.head;
        while pos != self.tail {
            let entry: &Entry = unsafe {
                &*(self.queue.sqes.add((pos & self.queue.ring_mask) as usize) as *const Entry)
            };
            d.entry(&entry);
            pos = pos.wrapping_add(1);
        }
        d.finish()
    }
}
