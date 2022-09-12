use std::{fs, io, os::windows::prelude::AsRawHandle};

use ioring_rs::{submit::Submitter, IoRing};

fn main() -> io::Result<()> {
    let infd = fs::File::create(file!())?;
    let outfd = fs::File::open("test.txt")?;
    let mut ring = IoRing::new(32)?;
    let submitter = ring.split().0;
    let size = infd.metadata()?.len();

    submitter.register_files_bufs(infd.as_raw_handle(), outfd.as_raw_handle())?;

    Ok(())
}
