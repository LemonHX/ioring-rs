use ioring_rs::windows::{
    win_ring, win_ring_cq_clear, win_ring_cqe_iter, win_ring_get_sqe, win_ring_prep_cancel,
    win_ring_prep_read, win_ring_queue_exit, win_ring_queue_init, win_ring_sqe_set_data64,
    win_ring_submit, win_ring_submit_and_wait, _NT_IORING_BUFFERREF, _NT_IORING_HANDLEREF,
    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
    _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
};
use std::{
    fs,
    io::{self, Write},
    os::windows::prelude::AsRawHandle,
    thread,
    time::Duration,
};
// use os_pipe::pipe;
unsafe fn clear_cqes(ring: *mut win_ring, string: &str) -> io::Result<()> {
    win_ring_submit_and_wait(ring, u32::MAX);
    for i in (*(*ring).info.__bindgen_anon_2.CompletionQueue).Head
        ..(*(*ring).info.__bindgen_anon_2.CompletionQueue).Tail
    {
        dbg!(i);
        let cqe = win_ring_cqe_iter(ring, i);
        dbg!(
            (*cqe).__bindgen_anon_1.ResultCode,
            (*cqe).Information,
            (*cqe).UserData,
            string
        );
    }
    win_ring_cq_clear(ring);

    Ok(())
}
fn main() -> std::io::Result<()> {
    unsafe {
        let (hReadPipe, mut hWritePipe) = os_pipe::pipe()?;
        let mut ring: win_ring = std::mem::zeroed();

        let thread_join_handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(2));
            println!("Writing pipe!");
            hWritePipe.write("Test Pipe".as_bytes()).unwrap();
            drop(hWritePipe);
        });

        win_ring_queue_init(32, &mut ring);
        let mut str = [0u8; 128];
        let mut ring_sqe = win_ring_get_sqe(&mut ring);

        win_ring_prep_read(
            ring_sqe,
            _NT_IORING_HANDLEREF {
                Handle: hReadPipe.as_raw_handle() as _,
            },
            _NT_IORING_BUFFERREF {
                Address: str.as_mut_ptr() as _,
            },
            128,
            0,
            _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
        );
        win_ring_sqe_set_data64(ring_sqe, 10);
        win_ring_submit(&mut ring);

        ring_sqe = win_ring_get_sqe(&mut ring);
        win_ring_prep_read(
            ring_sqe,
            _NT_IORING_HANDLEREF {
                Handle: hReadPipe.as_raw_handle() as _,
            },
            _NT_IORING_BUFFERREF {
                Address: str.as_mut_ptr() as _,
            },
            128,
            0,
            _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
        );
        win_ring_sqe_set_data64(ring_sqe, 20);
        win_ring_submit(&mut ring);

        ring_sqe = win_ring_get_sqe(&mut ring);
        win_ring_prep_cancel(
            ring_sqe,
            _NT_IORING_HANDLEREF {
                Handle: hReadPipe.as_raw_handle() as _,
            },
            0,
            _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
        );
        win_ring_sqe_set_data64(ring_sqe, 100);
        win_ring_submit(&mut ring);

        win_ring_queue_exit(&mut ring);

        clear_cqes(&mut ring, "read")?;
        thread_join_handle.join().unwrap();
        Ok(())
    }
}
