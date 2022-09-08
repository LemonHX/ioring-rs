use ioring_rs::windows::{
    win_ring, win_ring_cqe_iter, win_ring_get_sqe, win_ring_prep_read,
    win_ring_prep_register_buffers, win_ring_prep_register_files, win_ring_queue_exit,
    win_ring_queue_init, win_ring_sqe_set_data64, win_ring_submit_and_wait, IORING_BUFFER_INFO,
    IORING_REGISTERED_BUFFER, _NT_IORING_BUFFERREF, _NT_IORING_HANDLEREF,
    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_BUFFER,
    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_FILE, _NT_IORING_REG_FILES_FLAGS,
    _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
};
use std::{fs, io, os::windows::prelude::AsRawHandle};

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
    Ok(())
}
fn main() -> std::io::Result<()> {
    unsafe {
        // dbg!(std::mem::sizeof<_NT_IORING_SQ_FLAGS>());
        let f = fs::File::open("test.txt")?;
        let mut ring: win_ring = std::mem::zeroed();
        win_ring_queue_init(32, &mut ring);
        let mut buf4normal = [0u8; 32];
        let mut buf4fixed = [0u8; 32];
        let bufferInfo = IORING_BUFFER_INFO {
            Address: buf4fixed.as_mut_ptr() as *mut _,
            Length: 32,
        };
        let mut ring_sqe = win_ring_get_sqe(&mut ring);
        win_ring_prep_register_buffers(
            ring_sqe,
            &bufferInfo,
            1,
            std::mem::zeroed(),
            _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
        );
        win_ring_sqe_set_data64(ring_sqe, 140);
        win_ring_submit_and_wait(&mut ring, 1);
        clear_cqes(&mut ring, "register buffr")?;

        let mut ring_sqe = win_ring_get_sqe(&mut ring);
        win_ring_prep_register_files(
            ring_sqe,
            f.as_raw_handle() as _,
            1,
            std::mem::zeroed(),
            _NT_IORING_REG_FILES_REQ_FLAGS_NT_IORING_REG_FILES_REQ_FLAG_NONE,
        );
        win_ring_sqe_set_data64(ring_sqe, 140);
        win_ring_submit_and_wait(&mut ring, 1);
        clear_cqes(&mut ring, "register file")?;

        for x in 0..2 {
            ring_sqe = win_ring_get_sqe(&mut ring);
            if (x & 1) != 0 {
                let read_param = _NT_IORING_BUFFERREF {
                    FixedBuffer: IORING_REGISTERED_BUFFER {
                        BufferIndex: 0,
                        Offset: 0,
                    },
                };
                let file_param = _NT_IORING_HANDLEREF { Handle: 0 };
                win_ring_prep_read(
                    ring_sqe,
                    file_param,
                    read_param,
                    8,
                    0,
                    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_BUFFER
                        | _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_REGISTERED_FILE,
                );
            } else {
                win_ring_prep_read(
                    ring_sqe,
                    _NT_IORING_HANDLEREF {
                        Handle: f.as_raw_handle() as _,
                    },
                    _NT_IORING_BUFFERREF {
                        Address: buf4normal.as_mut_ptr() as _,
                    },
                    16,
                    0,
                    _NT_IORING_OP_FLAGS_NT_IORING_OP_FLAG_NONE,
                );
            }
            win_ring_sqe_set_data64(ring_sqe, x * 100);
        }
        win_ring_submit_and_wait(&mut ring, 1);

        clear_cqes(&mut ring, "read")?;
        dbg!(buf4normal, buf4fixed);
        win_ring_queue_exit(&mut ring);

        Ok(())
    }
}
