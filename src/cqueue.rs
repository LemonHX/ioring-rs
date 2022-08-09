use std::{io, os::windows::prelude::RawHandle};

use windows::Win32::Storage::FileSystem::IORING_INFO;

pub struct Inner {}

impl Inner {
    pub(crate) unsafe fn new(p: &IORING_INFO) -> io::Result<Inner> {
        todo!()
    }
}
