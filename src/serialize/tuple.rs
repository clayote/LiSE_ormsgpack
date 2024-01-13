// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;
use serde_bytes::ByteBuf;

pub struct TupleSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl TupleSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        TupleSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for TupleSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        let len = ffi!(PyTuple_GET_SIZE(self.ptr)) as usize;
        let mut seq = ser.serialize_seq(Some(len)).unwrap();
        if len > 0 {
            for i in 0..=len - 1 {
                let elem = nonnull!(ffi!(PyTuple_GET_ITEM(self.ptr, i as isize)));
                seq.serialize_element(&PyObjectSerializer::new(
                    elem.as_ptr(),
                    self.opts,
                    self.default_calls,
                    self.recursion + 1,
                    self.default,
                )).unwrap();
            }
        }
        let _ = seq.end();
        serializer.serialize_newtype_struct(
            rmp_serde::MSGPACK_EXT_STRUCT_NAME,
            &(0i8, ByteBuf::from(buf.buffer()))
        )
    }
}
