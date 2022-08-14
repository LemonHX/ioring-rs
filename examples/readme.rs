use ioring_rs::{opcode, IoRing};
use std::{fs, io, io::Write, mem, os::windows::prelude::RawHandle};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(1024)?;
    let mut buf = [0u8; 1024];
    let mut f = fs::File::create("test.txt")?;

    let mut entry = opcode::write(&mut ring, &mut f, &mut buf, 0)?;
    entry.set_offset(0);
    entry.set_len(buf.len() as u64);
    entry.set_flags(opcode::Flags::SYNC);
    ring.submit()?;
    Ok(())
}
