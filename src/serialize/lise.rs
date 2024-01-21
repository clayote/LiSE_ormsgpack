use crate::opt::Opt;
use crate::serialize::serializer::PyObjectSerializer;
use crate::typeref::{load_lise_types, LiSEType, LISE_TYPES, NONE};
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

pub struct ExceptionSerializer {
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

impl ExceptionSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        ExceptionSerializer {
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
    ($self: ident, $seq: ident, $name:ident) => {
        let _ = match $seq.serialize_element(&PyObjectSerializer::new(
            $name,
            $self.opts,
            $self.default_calls,
            $self.recursion + 1,
            $self.default,
        )) {
            Ok(ok) => ok,
            Err(e) => return Err(serde::ser::Error::custom(e))
        };
    };
}

impl Serialize for LiSESerializer {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let ob_type = ob_type!(self.ptr);
        let lise_type = LiSEType::from_ob_type(ob_type).unwrap();
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        Ok(match lise_type {
            LiSEType::Character => {
                let _ = match PyObjectSerializer::new(
                    getattr!(self.ptr, "name\0"),
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(&mut ser) {
                    Ok(this) => this,
                    Err(e) => return Err(serde::ser::Error::custom(e))
                };
                serializer.serialize_newtype_struct(
                    rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                    &(0x7fi8, ByteBuf::from(buf.buffer())),
                )?
            }
            LiSEType::Thing => {
                let graph: *mut pyo3::ffi::PyObject = getattr!(getattr!(self.ptr, "graph\0"), "name\0");
                let node: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "node\0");
                let mut seq = ser.serialize_seq(Some(2)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, node);
                let _ = seq.end();
                serializer.serialize_newtype_struct(
                    rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                    &(0x7di8, ByteBuf::from(buf.buffer())),
                )?
            }
            LiSEType::Place => {
                let graph: *mut pyo3::ffi::PyObject = getattr!(getattr!(self.ptr, "graph\0"), "name\0");
                let node: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "node\0");
                let mut seq = ser.serialize_seq(Some(2)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, node);
                let _ = seq.end();
                serializer.serialize_newtype_struct(
                    rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                    &(0x7ei8, ByteBuf::from(buf.buffer())),
                )?
            }
            LiSEType::Portal => {
            let graph: *mut pyo3::ffi::PyObject = getattr!(getattr!(self.ptr, "graph\0"), "name\0");
                let orig: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "orig\0");
                let dest: *mut pyo3::ffi::PyObject = getattr!(self.ptr, "dest\0");
                let mut seq = ser.serialize_seq(Some(3)).unwrap();
                seria!(self, seq, graph);
                seria!(self, seq, orig);
                seria!(self, seq, dest);
                let _ = seq.end();
                serializer.serialize_newtype_struct(
                    rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                    &(0x7ci8, ByteBuf::from(buf.buffer())),
                )?
            }
            LiSEType::FinalRule => serializer.serialize_newtype_struct(
                rmp_serde::MSGPACK_EXT_STRUCT_NAME,
                &(0x7bi8, ByteBuf::new()),
            )?,
        })
    }
}

impl Serialize for SetSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
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
        serializer.serialize_newtype_struct(
            rmp_serde::MSGPACK_EXT_STRUCT_NAME,
            &(0x02i8, ByteBuf::from(buf.buffer())),
        )
    }
}

impl Serialize for FrozenSetSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
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
        serializer.serialize_newtype_struct(
            rmp_serde::MSGPACK_EXT_STRUCT_NAME,
            &(0x01i8, ByteBuf::from(buf.buffer())),
        )
    }
}

impl Serialize for ExceptionSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let types = unsafe {
            LISE_TYPES
                .get_or_init(load_lise_types)
                .expect("Couldn't load LiSE types")
                .as_ref()
        };
        let args = getattr!(self.ptr, "args\0");
        let len = ffi!(PyTuple_GET_SIZE(args)) as usize;
        let mut buf = std::io::BufWriter::new(vec![]);
        let mut ser = rmp_serde::Serializer::new(&mut buf);
        let mut seq = ser.serialize_seq(Some(len + 2)).unwrap();
        let exc_cls = getattr!(self.ptr, "__class__\0");
        let exc_cls_name = getattr!(exc_cls, "__name__\0");
        seria!(self, seq, exc_cls_name);
        let tb = ffi!(PyException_GetTraceback(self.ptr));
        let tb_dict: *mut pyo3::ffi::PyObject;
        if tb.is_null() {
            tb_dict = unsafe { NONE };
        } else {
            let arg_list = ffi!(PyList_New(1));
            ffi!(PyList_SetItem(arg_list, 0, tb));
            let traceback = ffi!(PyObject_CallObject(types.traceback, arg_list));
            tb_dict = ffi!(PyObject_CallMethodNoArgs(
                traceback,
                pyo3::ffi::PyUnicode_FromString("to_dict\0".as_ptr() as *const c_char)
            ));
        }
        seria!(self, seq, tb_dict);
        if len > 0 {
            for i in 0..=len - 1 {
                let elem = nonnull!(ffi!(PyTuple_GET_ITEM(args, i as isize))).as_ptr();
                seria!(self, seq, elem);
            }
        }
        let _ = seq.end();
        serializer.serialize_newtype_struct(
            rmp_serde::MSGPACK_EXT_STRUCT_NAME,
            &(0x03i8, ByteBuf::from(buf.buffer())),
        )
    }
}
