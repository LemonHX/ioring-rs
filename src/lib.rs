#[macro_use]
mod squeue;
mod cqueue;
mod opcode;
mod register;
mod submit;
mod types;
use std::{
    io,
    mem::ManuallyDrop,
    os::windows::prelude::{AsRawHandle, RawHandle},
};
use windows::Win32::Storage::FileSystem::{
    CreateIoRing, IORING_CREATE_ADVISORY_FLAGS, IORING_CREATE_ADVISORY_FLAGS_NONE,
    IORING_CREATE_FLAGS, IORING_CREATE_REQUIRED_FLAGS_NONE, IORING_INFO, IORING_VERSION_3,
};
use xmmap::Mmap;
pub struct IoRing {
    sq: squeue::Inner,
    cq: cqueue::Inner,
    info: Info,
}

/// IoRing build info
#[derive(Clone, Default)]
pub struct Builder {
    dontfork: bool,
    info: IORING_INFO,
}

/// The Info that were used to construct an [`IoRing`].
#[derive(Clone)]
pub struct Info(IORING_INFO);

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
            info: IORING_INFO::default(),
        }
    }
    fn with_params(entries: u32, mut p: IORING_INFO) -> std::io::Result<IoRing> {
        p.SubmissionQueueSize = entries;
        p.CompletionQueueSize = entries * 2;
        let res = unsafe {
            CreateIoRing(
                IORING_VERSION_3,
                IORING_CREATE_FLAGS {
                    Required: IORING_CREATE_REQUIRED_FLAGS_NONE,
                    Advisory: IORING_CREATE_ADVISORY_FLAGS_NONE,
                },
                entries,
                entries * 2,
            )
            .expect("CreateIoRing failed")
        };
        #[inline]
        unsafe fn setup_queue(p: &IORING_INFO) -> io::Result<(squeue::Inner, cqueue::Inner)> {
            let sq = squeue::Inner::new(p)?;
            let cq = cqueue::Inner::new(p)?;

            Ok((sq, cq))
        }

        let (sq, cq) = unsafe { setup_queue(&p)? };
        Ok(IoRing {
            sq,
            cq,
            info: Info(p),
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
        self.submitter().submit_and_wait(want)
    }

    /// Get the submitter, submission queue and completion queue of the io_uring instance. This can
    /// be used to operate on the different parts of the io_uring instance independently.
    ///
    /// If you use this method to obtain `sq` and `cq`,
    /// please note that you need to `drop` or `sync` the queue before and after submit,
    /// otherwise the queue will not be updated.
    #[inline]
    pub fn split(&mut self) -> (Submitter<'_>, SubmissionQueue<'_>, CompletionQueue<'_>) {
        let submit = Submitter::new(
            &self.fd,
            &self.params,
            self.sq.head,
            self.sq.tail,
            self.sq.flags,
        );
        (submit, self.sq.borrow(), self.cq.borrow())
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
