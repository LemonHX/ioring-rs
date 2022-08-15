use std::{fmt, sync::atomic};

use windows::Win32::Storage::FileSystem::{IORING_CQE, IORING_INFO};

pub(crate) struct Inner {
    head: *const atomic::AtomicU32,
    tail: *const atomic::AtomicU32,
    ring_mask: u32,

    cqes: *const IORING_CQE,
    info: *const IORING_INFO,
}

/// An io_uring instance's completion queue. This stores all the I/O operations that have completed.
pub struct CompletionQueue<'a> {
    head: u32,
    tail: u32,
    queue: &'a Inner,
    info: &'a IORING_INFO,
}

/// An entry in the completion queue, representing a complete I/O operation.
#[repr(transparent)]
#[derive(Clone)]
pub struct Entry(pub(crate) IORING_CQE);

impl Inner {
    pub(crate) unsafe fn new(p: &IORING_INFO) -> Self {
        let head = p.CompletionQueue.as_mut().unwrap().Head as *const atomic::AtomicU32;
        let tail = p.CompletionQueue.as_mut().unwrap().Tail as *const atomic::AtomicU32;
        let ring_mask = p.CompletionQueueSizeMask;
        let cqes = p.CompletionQueue.as_mut().unwrap().Entries as *const IORING_CQE;
        let info = p;
        Self {
            head,
            tail,
            ring_mask,
            cqes,
            info,
        }
    }

    #[inline]
    pub(crate) unsafe fn borrow_shared(&self) -> CompletionQueue<'_> {
        CompletionQueue {
            head: *self.head.cast::<u32>(),
            tail: (*self.tail).load(atomic::Ordering::Acquire),
            queue: self,
            info: &*self.info,
        }
    }

    #[inline]
    pub(crate) fn borrow(&mut self) -> CompletionQueue<'_> {
        unsafe { self.borrow_shared() }
    }
}

impl CompletionQueue<'_> {
    /// Synchronize this type with the real completion queue.
    ///
    /// This will flush any entries consumed in this iterator and will make available new entries
    /// in the queue if the kernel has produced some entries in the meantime.
    #[inline]
    pub fn sync(&mut self) {
        unsafe {
            (*self.queue.head).store(self.head, atomic::Ordering::Release);
            self.tail = (*self.queue.tail).load(atomic::Ordering::Acquire);
        }
    }

    /// Get the total number of entries in the completion queue ring buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        // let view = (&self.queue.cqes) as *const _ as *const IORING_CQE;
        // let slice = unsafe { std::slice::from_raw_parts(view, mem::size_of::<IORING_CQE>()) };
        // slice.len() as usize
        todo!()
    }

    /// Returns `true` if there are no completion queue events to be processed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the completion queue is at maximum capacity. If
    /// [`is_feature_nodrop`](crate::Parameters::is_feature_nodrop) is not set, this will cause any
    /// new completion queue events to be dropped by the kernel.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    #[cfg(feature = "unstable")]
    #[inline]
    pub fn fill<'a>(&mut self, entries: &'a mut [MaybeUninit<Entry>]) -> &'a mut [Entry] {
        let len = std::cmp::min(self.len(), entries.len());

        for entry in &mut entries[..len] {
            *entry = MaybeUninit::new(Entry(unsafe {
                *self
                    .queue
                    .cqes
                    .add((self.head & self.queue.ring_mask) as usize)
            }));
            self.head = self.head.wrapping_add(1);
        }

        unsafe { std::slice::from_raw_parts_mut(entries as *mut _ as *mut Entry, len) }
    }
}

impl Drop for CompletionQueue<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe { &*self.queue.head }.store(self.head, atomic::Ordering::Release);
    }
}

impl Iterator for CompletionQueue<'_> {
    type Item = Entry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.head != self.tail {
            let entry = unsafe {
                *self
                    .queue
                    .cqes
                    .add((self.head & self.queue.ring_mask) as usize)
            };
            self.head = self.head.wrapping_add(1);
            Some(Entry(entry))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for CompletionQueue<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.tail.wrapping_sub(self.head) as usize
    }
}

impl Entry {
    /// The operation-specific result code. For example, for a [`Read`](crate::opcode::Read)
    /// operation this is equivalent to the return value of the `read(2)` system call.
    #[inline]
    pub fn result(&self) -> i32 {
        self.0.ResultCode.0
    }

    /// The user data of the request, as set by
    /// [`Entry::user_data`](crate::squeue::Entry::user_data) on the submission queue event.
    #[inline]
    pub fn user_data(&self) -> usize {
        self.0.UserData
    }

    /// Metadata related to the operation.
    ///
    /// This is currently used for:
    /// - Storing the selected buffer ID, if one was selected. See
    /// [`BUFFER_SELECT`](crate::squeue::Flags::BUFFER_SELECT) for more info.
    #[inline]
    pub fn Information(&self) -> usize {
        self.0.Information
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("result", &self.result())
            .field("user_data", &self.user_data())
            .field("Information", &self.Information())
            .finish()
    }
}

#[cfg(feature = "unstable")]
pub fn buffer_select(flags: u32) -> Option<u16> {
    if flags & sys::IORING_CQE_F_BUFFER != 0 {
        let id = flags >> sys::IORING_CQE_BUFFER_SHIFT;

        // FIXME
        //
        // Should we return u16? maybe kernel will change value of `IORING_CQE_BUFFER_SHIFT` in future.
        Some(id as u16)
    } else {
        None
    }
}
