use std::collections::VecDeque;
use std::net::TcpListener;
use std::{io, os::windows::prelude::RawHandle};
use ioring_rs::{opcode, squeue, IoRing};
use slab::Slab;

#[derive(Clone, Debug)]
enum Token {
    Accept,
    Poll {
        fd: RawHandle,
    },
    Read {
        fd: RawHandle,
        buf_index: usize,
    },
    Write {
        fd: RawHandle,
        buf_index: usize,
        offset: usize,
        len: usize,
    },
}
fn main(){
    
}