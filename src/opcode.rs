//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use crate::{
    squeue::Entry,
    windows::{
        win_ring, win_ring_get_sqe, win_ring_prep_nop, win_ring_prep_read,
        win_ring_prep_register_buffers, win_ring_prep_register_files, HANDLE, NT_IORING_BUFFERREF,
        NT_IORING_HANDLEREF, _IORING_BUFFER_INFO, _IORING_OP_CODE_IORING_OP_NOP,
        _IORING_OP_CODE_IORING_OP_READ, _IORING_OP_CODE_IORING_OP_REGISTER_FILES,
        _NT_IORING_OP_FLAGS, _NT_IORING_REG_BUFFERS_FLAGS, _NT_IORING_REG_FILES_FLAGS,
        _NT_IORING_SUBMISSION_QUEUE,
    },
};

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
    pub struct Nop {  ring:{* mut win_ring} ;; }

    pub const CODE = _IORING_OP_CODE_IORING_OP_NOP;

    pub fn build(self) -> Entry {
        let Nop { ring } = self;
        unsafe{
            let mut sqe = win_ring_get_sqe(ring);
            win_ring_prep_nop(sqe);
            Entry(sqe)
        }
    }
);

opcode!(
    /// Do not perform any I/O.
    ///
    /// This is useful for testing the performance of the io_uring implementation itself.
    #[derive(Debug)]
    pub struct Read {
        ring:{*mut win_ring},
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
            ring,
            file,
            buffer,
            size_to_read,
            file_offset,
            common_op_flags,
        } = self;

        unsafe {
            let mut sqe = win_ring_get_sqe(ring);
            win_ring_prep_read(sqe,
                file,
                buffer,
                size_to_read,
                file_offset,
                common_op_flags
            );
            Entry(sqe)
        }
    }
);

opcode!(
    /// This command is an alternative to using
    /// [`Submitter::register_files_update`](crate::Submitter::register_files_update) which then
    /// works in an async fashion, like the rest of the io_uring commands.
    pub struct RegisterFiles {
        ring:{*mut win_ring},
        handles :{ *const HANDLE},
        count:{u32},
        flags:{_NT_IORING_REG_FILES_FLAGS},
        common_op_flags:{_NT_IORING_OP_FLAGS}
     ;;
    }

    pub const CODE = _IORING_OP_CODE_IORING_OP_REGISTER_FILES;

    pub fn build(self) -> Entry {
        let RegisterFiles {
            ring,
            handles,
            count,
            flags,
            common_op_flags,
         } = self;

         unsafe{
            let mut sqe = win_ring_get_sqe(ring);
            win_ring_prep_register_files(sqe,
                handles,
                count,
                flags,
                common_op_flags
            );
            Entry(sqe)
         }
    }
);

opcode!(
    /// This command is an alternative to using
    /// [`Submitter::register_files_update`](crate::Submitter::register_files_update) which then
    /// works in an async fashion, like the rest of the io_uring commands.
    pub struct RegisterBuffers {
        ring:{*mut win_ring},
        handles :{ *const _IORING_BUFFER_INFO },
        count:{u32},
        flags:{_NT_IORING_REG_BUFFERS_FLAGS},
        common_op_flags:{_NT_IORING_OP_FLAGS}
     ;;
    }

    pub const CODE = _IORING_OP_CODE_IORING_OP_REGISTER_FILES;

    pub fn build(self) -> Entry {
        let RegisterBuffers {
            ring,
            handles,
            count,
            flags,
            common_op_flags,
         } = self;
         unsafe {
            let mut sqe = win_ring_get_sqe(ring);
            win_ring_prep_register_buffers(sqe,
                handles,
                count,
                flags,
                common_op_flags
            );

            Entry(sqe)
         }
    }
);
