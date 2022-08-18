use ioring_rs::{opcode, IoRing};
use std::{
    fs, io,
    io::Write,
    mem,
    os::windows::prelude::{AsRawHandle, RawHandle},
};
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{
        IORING_BUFFER_REF_0, IORING_HANDLE_REF_0, IORING_OP_FLAGS,
        IORING_OP_FLAG_REGISTERED_BUFFER, IORING_OP_FLAG_REGISTERED_FILE,
    },
};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(1024)?;
    let mut buf = [0u8; 1024];
    let mut f = fs::File::create("test.txt")?;
    let commonopflags = IORING_OP_FLAG_REGISTERED_FILE;

    let mut entry = opcode::Read::new(
        IORING_HANDLE_REF_0 {
            Handle: HANDLE(f.as_raw_handle() as _),
        },
        IORING_BUFFER_REF_0 {
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

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    Ok(())
}
