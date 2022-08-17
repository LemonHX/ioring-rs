//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use std::mem;

use crate::squeue::Entry;
use windows::Win32::Storage::FileSystem::{
    IORING_HANDLE_REF, IORING_OP_CODE, IORING_OP_NOP, IORING_OP_READ, IORING_SQE,
};
/// inline zeroed io improve codegen
#[inline(always)]
fn sqe_zeroed() -> IORING_SQE {
    unsafe { mem::zeroed() }
}

macro_rules! opcode {
    (@type $name:ty ) => {
        $name
    };
    (
        $( #[$outer:meta] )*
        pub struct $name:ident {
            $( #[$new_meta:meta] )*

            $( $field:ident : { $( $tnt:tt )+ } ),*

            $(,)?

            ;;

            $(
                $( #[$opt_meta:meta] )*
                $opt_field:ident : $opt_tname:ty = $default:expr
            ),*

            $(,)?
        }

        pub const CODE = $opcode:expr;

        $( #[$build_meta:meta] )*
        pub fn build($self:ident) -> Entry $build_block:block
    ) => {
        $( #[$outer] )*
        pub struct $name {
            $( $field : opcode!(@type $( $tnt )*), )*
            $( $opt_field : $opt_tname, )*
        }

        impl $name {
            $( #[$new_meta] )*
            #[inline]
            pub fn new($( $field : $( $tnt )* ),*) -> Self {
                $name {
                    $( $field: $field.into(), )*
                    $( $opt_field: $default, )*
                }
            }

            /// The opcode of the operation. This can be passed to
            /// [`Probe::is_supported`](crate::Probe::is_supported) to check if this operation is
            /// supported with the current kernel.
            pub const CODE: i32 = $opcode.0;

            $(
                $( #[$opt_meta] )*
                #[inline]
                pub const fn $opt_field(mut self, $opt_field: $opt_tname) -> Self {
                    self.$opt_field = $opt_field;
                    self
                }
            )*

            $( #[$build_meta] )*
            #[inline]
            pub fn build($self) ->Entry $build_block
        }
    }
}

opcode!(
    /// Do not perform any I/O.
    ///
    /// This is useful for testing the performance of the io_uring implementation itself.
    #[derive(Debug)]
    pub struct Nop { ;; }

    pub const CODE = IORING_OP_NOP;

    pub fn build(self) -> Entry {
        let Nop {} = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode = IORING_OP_CODE ( Self::CODE);
        Entry(sqe)
    }
);

opcode!(
    /// Do not perform any I/O.
    ///
    /// This is useful for testing the performance of the io_uring implementation itself.
    #[derive(Debug)]
    pub struct Read {
         file:IORING_HANDLE_REF,
       buffer:IORING_BUFFER_REF,
  sizeToRead:u32,
     fileOffset:u32,
     commonOpFlags:IORING_OP_FLAGS
     }

    pub const CODE = IORING_OP_READ;

    pub fn build(self) -> Entry {
        let Read {} = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode = IORING_OP_CODE ( Self::CODE);
        sqe.CommonOpFlags = ;
        Entry(sqe)
    }
);
