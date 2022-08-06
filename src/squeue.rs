use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::atomic;

// use crate::sys;
use xmmap::Mmap;

use bitflags::bitflags;

/// typedef struct _NT_IORING_SUBMISSION_QUEUE
/// {
///     /* 0x0000 */ uint32_t Head;
///     /* 0x0004 */ uint32_t Tail;
///     /* 0x0008 */ NT_IORING_SQ_FLAGS Flags;
///     /* 0x0010 */ NT_IORING_SQE Entries[];
/// } NT_IORING_SUBMISSION_QUEUE, * PNT_IORING_SUBMISSION_QUEUE; /* size: 0x0010 */

pub(crate) struct Inner{
    pub(crate) head: *const atomic::AtomicU32,
    pub(crate) tail: *const atomic::AtomicU32,
    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,
    pub(crate) flags: *const atomic::AtomicU32,
    dropped: *const atomic::AtomicU32,

    pub(crate) sqes: *mut  crate::windows::_NT_IORING_SQE ,
}

pub struct SubmissionQueue<'a>{
    head: u32,
    tail: u32,
    queue: &'a Inner
}


// typedef struct _NT_IORING_SQE
// {
//     /* 0x0000 */ IORING_OP_CODE OpCode;
//     /* 0x0004 */ NT_IORING_SQE_FLAGS Flags;
//     /* 0x0008 */ uint64_t UserData;
//     union
//     {
//         /* 0x0010 */ NT_IORING_OP_READ Read;
//         /* 0x0010 */ NT_IORING_OP_REGISTER_FILES RegisterFiles;
//         /* 0x0010 */ NT_IORING_OP_REGISTER_BUFFERS RegisterBuffers;
//         /* 0x0010 */ NT_IORING_OP_CANCEL Cancel;
//         /* 0x0010 */ NT_IORING_OP_WRITE Write;
//         /* 0x0010 */ NT_IORING_OP_FLUSH Flush;
//         /* 0x0010 */ NT_IORING_OP_RESERVED ReservedMaxSizePadding;
//     }; /* size: 0x0030 */
// } NT_IORING_SQE, * PNT_IORING_SQE; /* size: 0x0040 */    

/// An entry in the submission queue, representing a request for an I/O operation.
///
/// These can be created via the opcodes in [`opcode`](crate::opcode).
#[repr(transparent)]
#[derive(Clone)]
pub struct Entry(pub(crate) crate::windows::_NT_IORING_SQE);

impl Inner{
    pub(crate) unsafe fn new(
        
    ){
        todo!()
    }
}

/// An error pushing to the submission queue due to it being full.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PushError;

impl Display for PushError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("submission queue is full")
    }
}

impl Error for PushError {}

// impl Debug for Entry {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Entry")
//             .field("op_code", &self.0.opcode)
//             .field("flags", &self.0.flags)
//             .field("user_data", &self.0.user_data)
//             .finish()
//     }
// }