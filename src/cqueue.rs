use std::{fmt, io, sync::atomic};

use crate::windows::{
    win_ring, win_ring_cqe_get_data64, win_ring_cqe_iter, win_ring_submit_and_wait,
    _NT_IORING_COMPLETION_QUEUE, _NT_IORING_CQE, _NT_IORING_INFO,
};

pub(crate) struct Inner {
    ring_mask: u32,
    cqes: *mut _NT_IORING_COMPLETION_QUEUE,
    pub(crate) info: *mut win_ring,
}

/// An io_uring instance's completion queue. This stores all the I/O operations that have completed.
pub struct CompletionQueue<'a> {
    head: u32,
    tail: u32,
    queue: &'a mut Inner,
}

unsafe impl Send for CompletionQueue<'_> {}
unsafe impl Sync for CompletionQueue<'_> {}

/// An entry in the completion queue, representing a complete I/O operation.
#[repr(transparent)]
#[derive(Clone)]
pub struct Entry(pub(crate) *mut _NT_IORING_CQE);

impl Inner {
    pub(crate) unsafe fn new(p: *mut win_ring) -> Self {
        let _head = Box::new(atomic::AtomicU32::new(
            (*p).info
                .__bindgen_anon_2
                .CompletionQueue
                .as_ref()
                .unwrap()
                .Head,
        ));
        let _tail = Box::new(atomic::AtomicU32::new(
            (*p).info
                .__bindgen_anon_2
                .CompletionQueue
                .as_ref()
                .unwrap()
                .Tail,
        ));
        let ring_mask = (*p).info.CompletionQueueRingMask;
        let cqes = (*p).info.__bindgen_anon_2.CompletionQueue;
        Self {
            ring_mask,
            cqes,
            info: p,
        }
    }

    #[inline]
    pub(crate) unsafe fn borrow_shared(&self) -> CompletionQueue<'_> {
        CompletionQueue {
            head: self.cqes.as_ref().unwrap().Head,
            tail: self.cqes.as_ref().unwrap().Tail,
            queue: (self as *const Self as *mut Self).as_mut().unwrap(),
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
            self.head = self.queue.cqes.as_ref().unwrap().Head;
            self.tail = self.queue.cqes.as_ref().unwrap().Tail;
        }
    }

    /// Get the total number of entries in the completion queue ring buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { (*self.queue.info).info.CompletionQueueSize as usize }
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
    unsafe fn clear_cqes(ring: *mut win_ring, string: &str) -> io::Result<()> {
        win_ring_submit_and_wait(ring, u32::MAX);
        for i in (*(*ring).info.__bindgen_anon_2.CompletionQueue).Head
            ..(*(*ring).info.__bindgen_anon_2.CompletionQueue).Tail
        {
            dbg!(i);
            let cqe = win_ring_cqe_iter(ring, i);
            dbg!(
                (*cqe).__bindgen_anon_1.ResultCode,
                (*cqe).Information,
                (*cqe).UserData,
                string
            );
        }
        Ok(())
    }
}

impl Drop for CompletionQueue<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            Self::clear_cqes(self.queue.info, "drop cqe");
        }
    }
}

impl Iterator for CompletionQueue<'_> {
    type Item = Entry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // self.sync();
        self.head += 1;
        Some(Entry(unsafe {
            win_ring_cqe_iter(self.queue.info, self.head)
        }))
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
        unsafe { (*self.0).__bindgen_anon_1.ResultCode as _ }
    }
    /// The user data of the request, as set by
    /// [`Entry::user_data`](crate::squeue::Entry::user_data) on the submission queue event.
    #[inline]
    pub fn user_data(&self) -> u64 {
        unsafe { win_ring_cqe_get_data64(self.0) }
    }

    /// Metadata related to the operation.
    ///
    /// This is currently used for:
    /// - Storing the selected buffer ID, if one was selected. See
    /// [`BUFFER_SELECT`](crate::squeue::Flags::BUFFER_SELECT) for more info.
    #[inline]
    pub fn information(&self) -> usize {
        unsafe { (*self.0).Information as _ }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("user_data", &self.user_data())
            .field("Information", &self.information())
            .finish()
    }
}
