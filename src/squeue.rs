use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::os::windows::prelude::RawHandle;
use std::sync::atomic;
use std::{io, mem};

use windows::Win32::Storage::FileSystem::{IORING_INFO, IORING_SQE, IORING_SQE_FLAGS};
// use crate::sys;

use bitflags::bitflags;

pub(crate) struct Inner {
    pub(crate) head: *const atomic::AtomicU32,
    pub(crate) tail: *const atomic::AtomicU32,
    pub(crate) ring_mask: u32,
    pub(crate) flags: *const atomic::AtomicU32,
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

impl Inner {
    #[rustfmt::skip]
    pub(crate) unsafe fn new(
        p: &IORING_INFO,
    ) -> Self {
        let head = p.SubmissionQueue.as_mut().unwrap().Head as *const atomic::AtomicU32;
        let tail = p.SubmissionQueue.as_mut().unwrap().Tail as *const atomic::AtomicU32;
        let flags = p.SubmissionQueue.as_mut().unwrap().Flags.0 as *const atomic::AtomicU32;
        let ring_mask = p.SubmissionQueueSizeMask;
        let sqes = p.SubmissionQueue.as_mut().unwrap().Entries as *mut IORING_SQE;
        Self {
            head,
            tail,
            ring_mask,
            flags,
            sqes,
        }
    }

    #[inline]
    pub(crate) unsafe fn borrow_shared(&self) -> SubmissionQueue<'_> {
        SubmissionQueue {
            head: (*self.head).load(atomic::Ordering::Acquire),
            tail: (*self.tail).load(atomic::Ordering::Acquire),
            queue: self,
        }
    }

    #[inline]
    pub(crate) fn borrow(&mut self) -> SubmissionQueue<'_> {
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
            (*self.queue.tail).store(self.tail, atomic::Ordering::Release);
            self.head = (*self.queue.head).load(atomic::Ordering::Acquire);
        }
    }

    /// Get the total number of entries in the submission queue ring buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        let view = (&self.queue.sqes) as *const _ as *const IORING_SQE;
        let slice = unsafe { std::slice::from_raw_parts(view, mem::size_of::<IORING_SQE>()) };
        slice.len() as usize
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
    pub unsafe fn push(&mut self, Entry(entry): &Entry) -> Result<(), PushError> {
        if !self.is_full() {
            *self
                .queue
                .sqes
                .add((self.tail & self.queue.ring_mask) as usize) = *entry;
            self.tail = self.tail.wrapping_add(1);
            Ok(())
        } else {
            Err(PushError)
        }
    }

    /// Attempts to push several [entries](Entry) into the queue.
    /// If the queue does not have space for all of the entries, an error is returned.
    ///
    /// # Safety
    ///
    /// Developers must ensure that parameters of all the entries (such as buffer) are valid and
    /// will be valid for the entire duration of the operation, otherwise it may cause memory
    /// problems.
    #[cfg(feature = "unstable")]
    #[inline]
    pub unsafe fn push_multiple(&mut self, entries: &[Entry]) -> Result<(), PushError> {
        if self.capacity() - self.len() < entries.len() {
            return Err(PushError);
        }

        for Entry(entry) in entries {
            *self
                .queue
                .sqes
                .add((self.tail & self.queue.ring_mask) as usize) = *entry;
            self.tail = self.tail.wrapping_add(1);
        }

        Ok(())
    }
}

impl Drop for SubmissionQueue<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe { &*self.queue.tail }.store(self.tail, atomic::Ordering::Release);
    }
}

impl Entry {
    /// Set the submission event's [flags](Flags).
    #[inline]
    pub fn flags(mut self, flags: IORING_SQE_FLAGS) -> Entry {
        self.0.Flags = flags;
        self
    }

    /// Set the user data. This is an application-supplied value that will be passed straight
    /// through into the [completion queue entry](crate::cqueue::Entry::user_data).
    #[inline]
    pub fn user_data(mut self, user_data: u64) -> Entry {
        self.0.UserData = user_data;
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
        f.debug_struct("Entry")
            .field("op_code", &self.0.OpCode)
            .field("flags", &self.0.Flags)
            .field("user_data", &self.0.UserData)
            .finish()
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
