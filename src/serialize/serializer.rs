// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::ffi::*;
use crate::opt::*;
use crate::serialize::bytes::*;
use crate::serialize::dataclass::*;
use crate::serialize::datetime::*;
use crate::serialize::default::*;
use crate::serialize::dict::*;
use crate::serialize::ext::*;
use crate::serialize::int::*;
use crate::serialize::lise::{
    ExceptionSerializer, FrozenSetSerializer, LiSESerializer, SetSerializer,
};
use crate::serialize::list::*;
use crate::serialize::numpy::*;
use crate::serialize::str::*;
use crate::serialize::tuple::*;
use crate::serialize::uuid::*;
use crate::serialize::writer::*;
use crate::typeref::*;
use pyo3::ffi::PyExc_Exception;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub const RECURSION_LIMIT: u8 = 255;

pub fn serialize(
    ptr: *mut pyo3::ffi::PyObject,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
    opts: Opt,
) -> Result<NonNull<pyo3::ffi::PyObject>, String> {
    let mut buf = BytesWriter::default();
    let obj = PyObjectSerializer::new(ptr, opts, 0, 0, default);
    let mut ser = rmp_serde::Serializer::new(&mut buf);
    let res = obj.serialize(&mut ser);
    match res {
        Ok(_) => Ok(buf.finish()),
        Err(err) => {
            ffi!(_Py_Dealloc(buf.finish().as_ptr()));
            Err(err.to_string())
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObType {
    Str,
    Bytes,
    Int,
    Bool,
    None,
    Float,
    List,
    Dict,
    Datetime,
    Date,
    Time,
    Tuple,
    Uuid,
    Dataclass,
    NumpyScalar,
    NumpyArray,
    Pydantic,
    Enum,
    StrSubclass,
    Ext,
    Unknown,
    LiSE,
    Set,
    FrozenSet,
    Exception,
}

pub fn pyobject_to_obtype(obj: *mut pyo3::ffi::PyObject, opts: Opt) -> ObType {
    let ob_type = ob_type!(obj);
    if is_type!(ob_type, STR_TYPE) {
        ObType::Str
    } else if is_type!(ob_type, BYTES_TYPE) {
        ObType::Bytes
    } else if is_type!(ob_type, INT_TYPE)
        && (opts & PASSTHROUGH_BIG_INT == 0
            || ffi!(_PyLong_NumBits(obj)) <= {
                if pylong_is_positive(obj) {
                    64
                } else {
                    63
                }
            })
    {
        ObType::Int
    } else if is_type!(ob_type, BOOL_TYPE) {
        ObType::Bool
    } else if is_type!(ob_type, NONE_TYPE) {
        ObType::None
    } else if is_type!(ob_type, FLOAT_TYPE) {
        ObType::Float
    } else if is_type!(ob_type, LIST_TYPE) {
        ObType::List
    } else if is_type!(ob_type, DICT_TYPE) {
        ObType::Dict
    } else if is_type!(ob_type, DATETIME_TYPE) && opts & PASSTHROUGH_DATETIME == 0 {
        ObType::Datetime
    } else if ffi!(PySet_Check(obj)) != 0 {
        ObType::Set
    } else if ffi!(PyFrozenSet_Check(obj)) != 0 {
        ObType::FrozenSet
    } else if ffi!(PyObject_IsInstance(obj, PyExc_Exception)) != 0 {
        ObType::Exception
    } else {
        pyobject_to_obtype_unlikely(obj, opts)
    }
}

macro_rules! is_subclass {
    ($ob_type:expr, $flag:ident) => {
        unsafe { (((*$ob_type).tp_flags & pyo3::ffi::$flag) != 0) }
    };
}

#[inline(never)]
pub fn pyobject_to_obtype_unlikely(obj: *mut pyo3::ffi::PyObject, opts: Opt) -> ObType {
    let ob_type = ob_type!(obj);
    let lise_type = LiSEType::from_ob_type(ob_type);
    if lise_type.is_ok() {
        ObType::LiSE
    } else if is_type!(ob_type, DATE_TYPE) && opts & PASSTHROUGH_DATETIME == 0 {
        ObType::Date
    } else if is_type!(ob_type, TIME_TYPE) && opts & PASSTHROUGH_DATETIME == 0 {
        ObType::Time
    } else if is_type!(ob_type, TUPLE_TYPE) && opts & PASSTHROUGH_TUPLE == 0 {
        ObType::Tuple
    } else if is_type!(ob_type, UUID_TYPE) {
        ObType::Uuid
    } else if is_type!(ob_type!(ob_type), ENUM_TYPE) {
        ObType::Enum
    } else if opts & PASSTHROUGH_SUBCLASS == 0 && is_subclass!(ob_type, Py_TPFLAGS_UNICODE_SUBCLASS)
    {
        ObType::StrSubclass
    } else if opts & PASSTHROUGH_SUBCLASS == 0
        && is_subclass!(ob_type, Py_TPFLAGS_LONG_SUBCLASS)
        && (opts & PASSTHROUGH_BIG_INT == 0
            || ffi!(_PyLong_NumBits(obj)) <= {
                if pylong_is_positive(obj) {
                    64
                } else {
                    63
                }
            })
    {
        ObType::Int
    } else if opts & PASSTHROUGH_SUBCLASS == 0 && is_subclass!(ob_type, Py_TPFLAGS_LIST_SUBCLASS) {
        ObType::List
    } else if opts & PASSTHROUGH_SUBCLASS == 0 && is_subclass!(ob_type, Py_TPFLAGS_DICT_SUBCLASS) {
        ObType::Dict
    } else if opts & PASSTHROUGH_DATACLASS == 0 && pydict_contains!(ob_type, DATACLASS_FIELDS_STR) {
        ObType::Dataclass
    } else if opts & SERIALIZE_NUMPY != 0 && is_numpy_scalar(ob_type) {
        ObType::NumpyScalar
    } else if opts & SERIALIZE_NUMPY != 0 && is_numpy_array(ob_type) {
        ObType::NumpyArray
    } else if opts & SERIALIZE_PYDANTIC != 0
        && (pydict_contains!(ob_type, PYDANTIC_FIELDS_STR)
            || pydict_contains!(ob_type, PYDANTIC2_FIELDS_STR))
    {
        ObType::Pydantic
    } else if is_type!(ob_type, EXT_TYPE) {
        ObType::Ext
    } else {
        ObType::Unknown
    }
}

pub struct PyObjectSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    obtype: ObType,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl PyObjectSerializer {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        PyObjectSerializer {
            ptr: ptr,
            obtype: pyobject_to_obtype(ptr, opts),
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for PyObjectSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.obtype {
            ObType::Str => StrSerializer::new(self.ptr).serialize(serializer),
            ObType::Bytes => BytesSerializer::new(self.ptr).serialize(serializer),
            ObType::StrSubclass => StrSubclassSerializer::new(self.ptr).serialize(serializer),
            ObType::Int => IntSerializer::new(self.ptr).serialize(serializer),
            ObType::None => serializer.serialize_unit(),
            ObType::Float => serializer.serialize_f64(ffi!(PyFloat_AS_DOUBLE(self.ptr))),
            ObType::Bool => serializer.serialize_bool(unsafe { self.ptr == TRUE }),
            ObType::Datetime => DateTime::new(self.ptr, self.opts).serialize(serializer),
            ObType::Date => Date::new(self.ptr).serialize(serializer),
            ObType::Time => match Time::new(self.ptr, self.opts) {
                Ok(val) => val.serialize(serializer),
                Err(TimeError::HasTimezone) => err!(TIME_HAS_TZINFO),
            },
            ObType::Uuid => UUID::new(self.ptr).serialize(serializer),
            ObType::Dict => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                DictGenericSerializer::new(
                    self.ptr,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(serializer)
            }
            ObType::List => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                if unlikely!(ffi!(PyList_GET_SIZE(self.ptr)) == 0) {
                    serializer.serialize_seq(Some(0)).unwrap().end()
                } else {
                    ListSerializer::new(
                        self.ptr,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                }
            }
            ObType::Tuple => TupleSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Set => SetSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::FrozenSet => FrozenSetSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Exception => ExceptionSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Dataclass => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                DataclassGenericSerializer::new(
                    self.ptr,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(serializer)
            }
            ObType::Pydantic => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                let dict = ffi!(PyObject_GetAttr(self.ptr, DICT_STR));
                if unlikely!(dict.is_null()) {
                    err!(PYDANTIC_MUST_HAVE_DICT)
                } else {
                    ffi!(Py_DECREF(dict));
                    DataclassFastSerializer::new(
                        dict,
                        self.opts,
                        self.default_calls,
                        self.recursion,
                        self.default,
                    )
                    .serialize(serializer)
                }
            }
            ObType::LiSE => {
                if unlikely!(self.recursion == RECURSION_LIMIT) {
                    err!(RECURSION_LIMIT_REACHED)
                }
                LiSESerializer::new(
                    self.ptr,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(serializer)
            }
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(self.ptr, VALUE_STR));
                ffi!(Py_DECREF(value));
                PyObjectSerializer::new(
                    value,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                )
                .serialize(serializer)
            }
            ObType::NumpyArray => match NumpyArray::new(self.ptr) {
                Ok(val) => val.serialize(serializer),
                Err(PyArrayError::Malformed) => err!("numpy array is malformed"),
                Err(PyArrayError::NotContiguous) | Err(PyArrayError::UnsupportedDataType) => {
                    if self.default.is_none() {
                        err!("numpy array is not C contiguous; use ndarray.tolist() in default")
                    } else {
                        DefaultSerializer::new(
                            self.ptr,
                            self.opts,
                            self.default_calls,
                            self.recursion,
                            self.default,
                        )
                        .serialize(serializer)
                    }
                }
            },
            ObType::NumpyScalar => NumpyScalar::new(self.ptr).serialize(serializer),
            ObType::Ext => ExtSerializer::new(self.ptr).serialize(serializer),
            ObType::Unknown => DefaultSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
        }
    }
}
