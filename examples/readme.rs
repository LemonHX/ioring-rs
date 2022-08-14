use ioring_rs::{opcode, IoRing};
use std::{
    fs, io,
    io::Write,
    mem,
    os::windows::prelude::{AsRawHandle, RawHandle},
};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(1024)?;
    let mut buf = [0u8; 1024];
    let mut f = fs::File::create("test.txt")?;

    let mut entry = opcode::Read::new(f.as_raw_handle(), buf.as_mut_ptr(), buf.len() as _)
        .build()
        .user_data(0x42);
    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission()
            .push(&entry)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    Ok(())
}
