//! Operation codes that can be used to construct [`squeue::Entry`](crate::squeue::Entry)s.

#![allow(clippy::new_without_default)]

use std::{fmt, io, io::Write, mem, os::windows::prelude::RawHandle, sync::atomic};

use crate::squeue::Entry;
