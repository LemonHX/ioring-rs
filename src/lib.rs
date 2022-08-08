#[macro_use]
mod squeue;
mod cqueue;
mod opcode;
mod register;
mod submit;
mod types;
use std::{
    mem::ManuallyDrop,
    os::windows::prelude::{AsRawHandle, RawHandle},
};
use xmmap::Mmap;
pub struct IoRing {
    sq: squeue::Inner,
    cq: cqueue::Inner,
    fd: RawHandle,
    info: Info,
    memory: ManuallyDrop<MemoryMap>,
}

#[allow(dead_code)]
struct MemoryMap {
    sq_mmap: Mmap,
    sqe_mmap: Mmap,
    cq_mmap: Option<Mmap>,
}

/// IoRing build info
#[derive(Clone, Default)]
pub struct Builder {
    dontfork: bool,
    info: windows::Win32::Storage::FileSystem::IORING_INFO,
}

/// The Info that were used to construct an [`IoRing`].
#[derive(Clone)]
pub struct Info(windows::Win32::Storage::FileSystem::IORING_INFO);

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
            info: windows::Win32::Storage::FileSystem::IORING_INFO::default(),
        }
    }
    fn with_params(
        entries: u32,
        mut p: windows::Win32::Storage::FileSystem::IORING_INFO,
    ) -> std::io::Result<IoRing> {
        p.SubmissionQueueSize = entries;
        p.CompletionQueueSize = entries * 2;
        todo!()
    }
}
