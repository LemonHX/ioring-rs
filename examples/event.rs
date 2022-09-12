use ioring_rs::{opcode, IoRing};
use std::{fs, io, os::windows::prelude::AsRawHandle, ptr};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{CreateEventA, WaitForSingleObject};

fn main() -> io::Result<()> {
    let mut ring = IoRing::new(32)?;
    let event: HANDLE = unsafe {
        CreateEventA(
            ptr::null_mut(),
            false,
            false,
            windows::core::PCSTR(ptr::null()),
        )
        .map_err(|e| {
            let description = format!("Failed to create event {}", e);
            io::Error::new(io::ErrorKind::Other, description)
        })?
    };
    let entry_reg_event = opcode::RegisterEvents::new(ring.info.0, event.0 as *mut _).build();

    unsafe {
        ring.submission()
            .push(&entry_reg_event)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;
    unsafe {
        if WaitForSingleObject(event, u32::MAX) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to wait for event",
            ));
        }
        match ring.peek_cqe() {
            Some(_) => {}
            None => {
                return Err(io::Error::new(io::ErrorKind::Other, "no cqe"));
            }
        };
    };

    Ok(())
}
