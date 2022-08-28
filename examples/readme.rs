use ioring_rs::windows::{
    _NT_IORING_BUFFERREF, _NT_IORING_HANDLEREF, _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
    _NT_IORING_REG_FILES_FLAGS,
};
use ioring_rs::{opcode, IoRing};
use std::{fs, io, os::windows::prelude::AsRawHandle};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(32)?;
    let mut buf = [0u8; 32];
    let f = fs::File::open("test.txt")?;
    let commonopflags = _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE;

    let entry_reg_file = opcode::RegisterFiles::new(
        f.as_raw_handle() as _,
        1,
        _NT_IORING_REG_FILES_FLAGS {
            Required: 0,
            Advisory: 0,
        },
        commonopflags,
    )
    .build()
    .user_data(140);

    unsafe {
        ring.submission()
            .push(&entry_reg_file)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(10)?;
    let mut cqe = ring.completion().next().unwrap();
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());
    assert_eq!(cqe.user_data(), 140);

    let entry_reg_buf = opcode::RegisterBuffers::new(
        buf.as_ptr() as _,
        1,
        _NT_IORING_REG_FILES_FLAGS {
            Required: 0,
            Advisory: 0,
        },
        commonopflags,
    )
    .build();

    unsafe {
        ring.submission()
            .push(&entry_reg_buf)
            .expect("submission queue is full");
    }
    ring.submit_and_wait(10)?;
    cqe = ring.completion().next().unwrap();
    assert!(cqe.result() == 0, "read error: {}", cqe.result());

    let entry_read = opcode::Read::new(
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
    .user_data(100);

    unsafe {
        ring.submission()
            .push(&entry_read)
            .expect("submission queue is full");
    }
    ring.submit_and_wait(10)?;

    cqe = ring.completion().next().expect("completion queue is empty");
    dbg!(buf);

    assert_eq!(cqe.user_data(), 100);
    assert!(cqe.result() == 0, "read error: {}", cqe.result());

    Ok(())
}
