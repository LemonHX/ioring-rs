//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use std::{mem, os::windows::prelude::RawHandle};

use crate::squeue::Entry;
use windows::Win32::Storage::FileSystem::{
    IORING_BUFFER_REF_0, IORING_HANDLE_REF_0, IORING_OP_CODE, IORING_OP_FLAGS, IORING_OP_NOP,
    IORING_OP_READ, IORING_OP_REGISTER_FILES, IORING_REG_FILES_FLAGS, IORING_SQE,
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
        file:{IORING_HANDLE_REF_0},
        buffer:{IORING_BUFFER_REF_0},
        size_to_read:{u32},
        file_offset:{u64},
        common_op_flags:{IORING_OP_FLAGS}
     ;;
     }

    pub const CODE = IORING_OP_READ;

    pub fn build(self) -> Entry {
        let Read {
            file,
            buffer,
            size_to_read,
            file_offset,
            common_op_flags,
        } = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode = IORING_OP_CODE ( Self::CODE);
        sqe.Op.Read. CommonOpFlags = common_op_flags;
        sqe.Op.Read.File =file;
        sqe.Op.Read.Buffer =buffer;
        sqe.Op.Read.Offset = file_offset;
        sqe.Op.Read.Length =size_to_read;
        Entry(sqe)
    }
);

opcode!(
    /// This command is an alternative to using
    /// [`Submitter::register_files_update`](crate::Submitter::register_files_update) which then
    /// works in an async fashion, like the rest of the io_uring commands.
    pub struct RegisterFiles {
        handles :{ *mut RawHandle},
        count:{u32},
        flags:{IORING_REG_FILES_FLAGS},
        common_op_flags:{IORING_OP_FLAGS}
     ;;
    }

    pub const CODE = IORING_OP_REGISTER_FILES;

    pub fn build(self) -> Entry {
        let RegisterFiles {
            handles,
            count,
            flags,
            common_op_flags,
         } = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode = IORING_OP_CODE ( Self::CODE);
        sqe.Op.RegisterFiles.Files = handles as * mut _;
        sqe.Op.RegisterFiles.CommonOpFlags =common_op_flags;
        sqe.Op.RegisterFiles.Count = count;
        sqe.Op.RegisterFiles.Flags = flags;
        Entry(sqe)
    }
);
