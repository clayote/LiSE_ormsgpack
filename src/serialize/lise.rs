use crate::opt::Opt;
use crate::serialize::serializer::PyObjectSerializer;
use crate::typeref::LiSEType;
use serde::ser::{Serialize, SerializeSeq};
use serde::Serializer;
use serde_bytes::ByteBuf;
use std::os::raw::c_char;
use std::ptr::NonNull;

pub struct LiSESerializer {
    pub ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

pub struct SetSerializer {
    pub ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

pub struct FrozenSetSerializer {
    pub ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl LiSESerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        LiSESerializer {
            ptr,
            opts,
            default_calls,
            recursion,
            default,
        }
    }
}

impl SetSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        SetSerializer {
            ptr,
            opts,
            default_calls,
            recursion,
            default,
        }
    }
}

impl FrozenSetSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        FrozenSetSerializer {
            ptr,
            opts,
            default_calls,
            recursion,
            default,
        }
    }
}

macro_rules! getattr {
    ($ptr:expr, $s:literal) => {
        ffi!(PyObject_GetAttrString($ptr, $s.as_ptr() as *const c_char))
    };
}

macro_rules! seria {
    ($self: ident, $seq: ident, $name:ident) => {$seq.serialize_element(&PyObjectSerializer::new(
                    $name,
                    $self.opts,
                    $self.default_calls,
                    $self.recursion + 1,
                    $self.default,
                )).unwrap()};
}

macro_rules! newtyp {
    ($serializer:ident, $code:literal, $buf:ident) => {
        $serializer.serialize_newtype_struct(
            rmp_serde::MSGPACK_EXT_STRUCT_NAME,
            &($code as i8, ByteBuf::from($buf.buffer()))
        )?
    };
}

impl Serialize for LiSESerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);
        let lise_type = LiSEType::from_ob_type(ob_type).unwrap();
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        Ok(match lise_type {
            LiSEType::Character => {
                PyObjectSerializer::new(
                    getattr!(self.ptr, "name\0"),
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default
                ).serialize(&mut ser).unwrap();
                newtyp!(serializer, 0x7f, buf)
            }
            LiSEType::Thing => {
                let graph: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "graph\0");
                let node: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "node\0");
                let mut seq = ser.serialize_seq(Some(2)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, node);
                let _ = seq.end();
                newtyp!(serializer, 0x7d, buf)
            }
            LiSEType::Place => {
                let graph: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "graph\0");
                let node: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "node\0");
                let mut seq = ser.serialize_seq(Some(2)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, node);
                let _ = seq.end();
                newtyp!(serializer, 0x7e, buf)
            }
            LiSEType::Portal => {
                let graph: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "graph\0");
                let orig: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "orig\0");
                let dest: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "dest\0");
                let mut seq = ser.serialize_seq(Some(3)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, orig);
                seria!(self, seq, dest);
                let _ = seq.end();
                newtyp!(serializer, 0x7c, buf)
            }
            LiSEType::FinalRule => serializer.serialize_newtype_struct(
                rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                &(0x7bi8, ByteBuf::new()),
            )?,
        })
    }
}

impl Serialize for SetSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        let len = ffi!(PySet_GET_SIZE(self.ptr)) as usize;
        let mut seq = ser.serialize_seq(Some(len)).unwrap();
        let it = ffi!(PyObject_GetIter(self.ptr));
        if len > 0 {
            for _ in 0..=len - 1 {
                let elem = nonnull!(ffi!(PyIter_Next(it)));
                seq.serialize_element(&PyObjectSerializer::new(
                    elem.as_ptr(),
                    self.opts,
                    self.default_calls,
                    self.recursion + 1,
                    self.default,
                ))
                    .unwrap();
            }
        }
        let _ = seq.end();
        newtyp!(serializer, 0x02, buf)
    }
}

impl Serialize for FrozenSetSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        let len = ffi!(PySet_GET_SIZE(self.ptr)) as usize;
        let mut seq = ser.serialize_seq(Some(len)).unwrap();
        let it = ffi!(PyObject_GetIter(self.ptr));
        if len > 0 {
            for _ in 0..=len - 1 {
                let elem = nonnull!(ffi!(PyIter_Next(it)));
                seq.serialize_element(&PyObjectSerializer::new(
                    elem.as_ptr(),
                    self.opts,
                    self.default_calls,
                    self.recursion + 1,
                    self.default,
                ))
                    .unwrap();
            }
        }
        let _ = seq.end();
        newtyp!(serializer, 0x01, buf)
    }
}
