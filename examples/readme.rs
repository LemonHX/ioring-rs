use ioring_rs::windows::{
    _NT_IORING_BUFFERREF, _NT_IORING_HANDLEREF,
    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_FILE,
};
use ioring_rs::{opcode, IoRing};
use std::{fs, io, os::windows::prelude::AsRawHandle};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(32)?;
    let mut buf = [0u8; 32];
    let f = fs::File::create("test.txt")?;
    let commonopflags = _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_FILE;

    let entry = opcode::Read::new(
        _NT_IORING_HANDLEREF {
            Handle: f.as_raw_handle() as _,
        },
        _NT_IORING_BUFFERREF {
            Address: buf.as_mut_ptr() as _,
        },
        buf.len() as _,
        0,
        commonopflags,
    )
    .build()
    .user_data(0x42);
    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission()
            .push(&entry)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(10)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    Ok(())
}
