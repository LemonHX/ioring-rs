//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use std::{mem, os::windows::prelude::RawHandle};

use crate::{squeue::Entry, windows::{_IORING_OP_CODE_IORING_OP_NOP, _NT_IORING_HANDLEREF, HANDLE,_NT_IORING_OP_FLAGS, _NT_IORING_SQE, _IORING_OP_CODE_IORING_OP_READ, NT_IORING_HANDLEREF, NT_IORING_BUFFERREF, _NT_IORING_REG_FILES_FLAGS, _IORING_OP_CODE_IORING_OP_REGISTER_FILES}};

/// inline zeroed io improve codegen
#[inline(always)]
fn sqe_zeroed() -> _NT_IORING_SQE {
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
            pub const CODE: i32 = $opcode;

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

    pub const CODE = _IORING_OP_CODE_IORING_OP_NOP;

    pub fn build(self) -> Entry {
        let Nop {} = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode =  Self::CODE;
        Entry(sqe)
    }
);

opcode!(
    /// Do not perform any I/O.
    ///
    /// This is useful for testing the performance of the io_uring implementation itself.
    #[derive(Debug)]
    pub struct Read {
        file:{NT_IORING_HANDLEREF},
        buffer:{NT_IORING_BUFFERREF},
        size_to_read:{u32},
        file_offset:{u64},
        common_op_flags:{_NT_IORING_OP_FLAGS}
     ;;
     }

    pub const CODE = _IORING_OP_CODE_IORING_OP_READ;

    pub fn build(self) -> Entry {
        let Read {
            file,
            buffer,
            size_to_read,
            file_offset,
            common_op_flags,
        } = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode =  Self::CODE;
        sqe.__bindgen_anon_1.Read. CommonOpFlags = common_op_flags;
        sqe.__bindgen_anon_1.Read.File =file;
        sqe.__bindgen_anon_1.Read.Buffer =buffer;
        sqe.__bindgen_anon_1.Read.Offset = file_offset;
        sqe.__bindgen_anon_1.Read.Length =size_to_read;
        Entry(sqe)
    }
);

opcode!(
    /// This command is an alternative to using
    /// [`Submitter::register_files_update`](crate::Submitter::register_files_update) which then
    /// works in an async fashion, like the rest of the io_uring commands.
    pub struct RegisterFiles {
        handles :{ *const HANDLE},
        count:{u32},
        flags:{_NT_IORING_REG_FILES_FLAGS},
        common_op_flags:{_NT_IORING_OP_FLAGS}
     ;;
    }

    pub const CODE = _IORING_OP_CODE_IORING_OP_REGISTER_FILES;

    pub fn build(self) -> Entry {
        let RegisterFiles {
            handles,
            count,
            flags,
            common_op_flags,
         } = self;

        let mut sqe = sqe_zeroed();
        sqe.OpCode =  Self::CODE;
        sqe.__bindgen_anon_1.RegisterFiles.__bindgen_anon_1.Handles = handles as * mut _;
        sqe.__bindgen_anon_1.RegisterFiles.CommonOpFlags =common_op_flags;
        sqe.__bindgen_anon_1.RegisterFiles.Count = count;
        sqe.__bindgen_anon_1.RegisterFiles.Flags = flags;
        Entry(sqe)
    }
);
