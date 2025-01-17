// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use ahash::RandomState;
use once_cell::race::OnceBox;
use pyo3::exceptions::PyTypeError;
use pyo3::ffi::*;
use pyo3::PyResult;
use std::os::raw::c_char;
use std::ptr::{null_mut, NonNull};
use std::sync::Once;

use crate::ext::create_ext_type;

pub struct NumpyTypes {
    pub array: *mut PyTypeObject,
    pub float64: *mut PyTypeObject,
    pub float32: *mut PyTypeObject,
    pub int64: *mut PyTypeObject,
    pub int32: *mut PyTypeObject,
    pub int16: *mut PyTypeObject,
    pub int8: *mut PyTypeObject,
    pub uint64: *mut PyTypeObject,
    pub uint32: *mut PyTypeObject,
    pub uint16: *mut PyTypeObject,
    pub uint8: *mut PyTypeObject,
    pub bool_: *mut PyTypeObject,
}

pub struct LiSETypes {
    pub character: *mut PyTypeObject,
    pub character_proxy: *mut PyTypeObject,
    pub thing: *mut PyTypeObject,
    pub thing_proxy: *mut PyTypeObject,
    pub place: *mut PyTypeObject,
    pub place_proxy: *mut PyTypeObject,
    pub portal: *mut PyTypeObject,
    pub portal_proxy: *mut PyTypeObject,
    pub final_rule: *mut PyTypeObject,
    pub traceback: *mut PyObject,
}

pub enum LiSEType {
    Character,
    Thing,
    Place,
    Portal,
    FinalRule,
}

impl LiSEType {
    pub fn from_ob_type(ob_type: *mut PyTypeObject) -> PyResult<LiSEType> {
        let types = unsafe {
            LISE_TYPES
                .get_or_init(load_lise_types)
                .expect("Couldn't load LiSE types")
                .as_ref()
        };
        if ob_type == types.character || ob_type == types.character_proxy {
            return Ok(LiSEType::Character);
        } else if ob_type == types.thing || ob_type == types.thing_proxy {
            return Ok(LiSEType::Thing);
        } else if ob_type == types.place || ob_type == types.place_proxy {
            return Ok(LiSEType::Place);
        } else if ob_type == types.portal || ob_type == types.portal_proxy {
            return Ok(LiSEType::Portal);
        } else if ob_type == types.final_rule {
            return Ok(LiSEType::FinalRule);
        }
        Err(PyTypeError::new_err("Unknown type"))
    }
}

pub static mut DEFAULT: *mut PyObject = null_mut();
pub static mut EXT_HOOK: *mut PyObject = null_mut();
pub static mut OPTION: *mut PyObject = null_mut();

pub static mut NONE: *mut PyObject = null_mut();
pub static mut TRUE: *mut PyObject = null_mut();
pub static mut FALSE: *mut PyObject = null_mut();
pub static mut EMPTY_UNICODE: *mut PyObject = null_mut();

pub static mut BYTES_TYPE: *mut PyTypeObject = null_mut();
pub static mut BYTEARRAY_TYPE: *mut PyTypeObject = null_mut();
pub static mut MEMORYVIEW_TYPE: *mut PyTypeObject = null_mut();
pub static mut STR_TYPE: *mut PyTypeObject = null_mut();
pub static mut INT_TYPE: *mut PyTypeObject = null_mut();
pub static mut BOOL_TYPE: *mut PyTypeObject = null_mut();
pub static mut NONE_TYPE: *mut PyTypeObject = null_mut();
pub static mut FLOAT_TYPE: *mut PyTypeObject = null_mut();
pub static mut LIST_TYPE: *mut PyTypeObject = null_mut();
pub static mut DICT_TYPE: *mut PyTypeObject = null_mut();
pub static mut DATETIME_TYPE: *mut PyTypeObject = null_mut();
pub static mut DATE_TYPE: *mut PyTypeObject = null_mut();
pub static mut TIME_TYPE: *mut PyTypeObject = null_mut();
pub static mut TUPLE_TYPE: *mut PyTypeObject = null_mut();
pub static mut UUID_TYPE: *mut PyTypeObject = null_mut();
pub static mut ENUM_TYPE: *mut PyTypeObject = null_mut();
pub static mut FIELD_TYPE: *mut PyTypeObject = null_mut();
pub static mut EXT_TYPE: *mut PyTypeObject = null_mut();
pub static mut EXCEPTION_TYPE: *mut PyTypeObject = null_mut();
pub static mut NUMPY_TYPES: OnceBox<Option<NonNull<NumpyTypes>>> = OnceBox::new();
pub static mut LISE_TYPES: OnceBox<Option<NonNull<LiSETypes>>> = OnceBox::new();
pub static mut UTCOFFSET_METHOD_STR: *mut PyObject = null_mut();
pub static mut NORMALIZE_METHOD_STR: *mut PyObject = null_mut();
pub static mut CONVERT_METHOD_STR: *mut PyObject = null_mut();
pub static mut DST_STR: *mut PyObject = null_mut();

pub static mut DICT_STR: *mut PyObject = null_mut();
pub static mut DATACLASS_FIELDS_STR: *mut PyObject = null_mut();
pub static mut SLOTS_STR: *mut PyObject = null_mut();
pub static mut PYDANTIC_FIELDS_STR: *mut PyObject = null_mut();
pub static mut PYDANTIC2_FIELDS_STR: *mut PyObject = null_mut();
pub static mut FIELD_TYPE_STR: *mut PyObject = null_mut();
pub static mut ARRAY_STRUCT_STR: *mut PyObject = null_mut();
pub static mut VALUE_STR: *mut PyObject = null_mut();
pub static mut INT_ATTR_STR: *mut PyObject = null_mut();

pub static mut HASH_BUILDER: OnceBox<ahash::RandomState> = OnceBox::new();

pub fn ahash_init() -> Box<ahash::RandomState> {
    unsafe {
        debug_assert!(!VALUE_STR.is_null());
        debug_assert!(!DICT_TYPE.is_null());
        debug_assert!(!STR_TYPE.is_null());
        debug_assert!(!BYTES_TYPE.is_null());
        Box::new(RandomState::with_seeds(
            VALUE_STR as u64,
            DICT_TYPE as u64,
            STR_TYPE as u64,
            BYTES_TYPE as u64,
        ))
    }
}

#[allow(non_upper_case_globals)]
pub static mut MsgpackEncodeError: *mut PyObject = null_mut();
#[allow(non_upper_case_globals)]
pub static mut MsgpackDecodeError: *mut PyObject = null_mut();

static INIT: Once = Once::new();

#[cold]
pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        assert!(crate::deserialize::KEY_MAP
            .set(crate::deserialize::KeyMap::default())
            .is_ok());
        PyDateTime_IMPORT();
        EXCEPTION_TYPE = (*PyExc_Exception).ob_type;
        NONE = Py_None();
        TRUE = Py_True();
        FALSE = Py_False();
        EMPTY_UNICODE = PyUnicode_New(0, 255);
        STR_TYPE = (*EMPTY_UNICODE).ob_type;
        BYTES_TYPE = (*PyBytes_FromStringAndSize("".as_ptr() as *const c_char, 0)).ob_type;

        {
            let bytearray = PyByteArray_FromStringAndSize("".as_ptr() as *const c_char, 0);
            BYTEARRAY_TYPE = (*bytearray).ob_type;

            let memoryview = PyMemoryView_FromObject(bytearray);
            MEMORYVIEW_TYPE = (*memoryview).ob_type;
            Py_DECREF(memoryview);
            Py_DECREF(bytearray);
        }

        DICT_TYPE = (*PyDict_New()).ob_type;
        LIST_TYPE = (*PyList_New(0)).ob_type;
        TUPLE_TYPE = (*PyTuple_New(0)).ob_type;
        NONE_TYPE = (*NONE).ob_type;
        BOOL_TYPE = (*TRUE).ob_type;
        INT_TYPE = (*PyLong_FromLongLong(0)).ob_type;
        FLOAT_TYPE = (*PyFloat_FromDouble(0.0)).ob_type;
        DATETIME_TYPE = look_up_datetime_type();
        DATE_TYPE = look_up_date_type();
        TIME_TYPE = look_up_time_type();
        UUID_TYPE = look_up_uuid_type();
        ENUM_TYPE = look_up_enum_type();
        FIELD_TYPE = look_up_field_type();
        EXT_TYPE = create_ext_type();
        INT_ATTR_STR = PyUnicode_InternFromString("int\0".as_ptr() as *const c_char);
        UTCOFFSET_METHOD_STR = PyUnicode_InternFromString("utcoffset\0".as_ptr() as *const c_char);
        NORMALIZE_METHOD_STR = PyUnicode_InternFromString("normalize\0".as_ptr() as *const c_char);
        CONVERT_METHOD_STR = PyUnicode_InternFromString("convert\0".as_ptr() as *const c_char);
        DST_STR = PyUnicode_InternFromString("dst\0".as_ptr() as *const c_char);
        DICT_STR = PyUnicode_InternFromString("__dict__\0".as_ptr() as *const c_char);
        DATACLASS_FIELDS_STR =
            PyUnicode_InternFromString("__dataclass_fields__\0".as_ptr() as *const c_char);
        SLOTS_STR = PyUnicode_InternFromString("__slots__\0".as_ptr() as *const c_char);
        PYDANTIC_FIELDS_STR = PyUnicode_InternFromString("__fields__\0".as_ptr() as *const c_char);
        PYDANTIC2_FIELDS_STR =
            PyUnicode_InternFromString("model_fields\0".as_ptr() as *const c_char);
        FIELD_TYPE_STR = PyUnicode_InternFromString("_field_type\0".as_ptr() as *const c_char);
        ARRAY_STRUCT_STR =
            PyUnicode_InternFromString("__array_struct__\0".as_ptr() as *const c_char);
        VALUE_STR = PyUnicode_InternFromString("value\0".as_ptr() as *const c_char);
        DEFAULT = PyUnicode_InternFromString("default\0".as_ptr() as *const c_char);
        EXT_HOOK = PyUnicode_InternFromString("ext_hook\0".as_ptr() as *const c_char);
        OPTION = PyUnicode_InternFromString("option\0".as_ptr() as *const c_char);
        MsgpackEncodeError = PyExc_TypeError;
        MsgpackDecodeError = PyExc_ValueError;

        HASH_BUILDER.get_or_init(ahash_init);
    });
}

#[cold]
unsafe fn look_up_numpy_type(numpy_module: *mut PyObject, np_type: &str) -> *mut PyTypeObject {
    let mod_dict = PyObject_GenericGetDict(numpy_module, null_mut());
    let ptr = PyMapping_GetItemString(mod_dict, np_type.as_ptr() as *const c_char);
    Py_XDECREF(ptr);
    Py_XDECREF(mod_dict);
    ptr as *mut PyTypeObject
}

#[cold]
pub fn load_numpy_types() -> Box<Option<NonNull<NumpyTypes>>> {
    unsafe {
        let numpy = PyImport_ImportModule("numpy\0".as_ptr() as *const c_char);
        if numpy.is_null() {
            PyErr_Clear();
            return Box::new(None);
        }

        let types = Box::new(NumpyTypes {
            array: look_up_numpy_type(numpy, "ndarray\0"),
            float32: look_up_numpy_type(numpy, "float32\0"),
            float64: look_up_numpy_type(numpy, "float64\0"),
            int8: look_up_numpy_type(numpy, "int8\0"),
            int16: look_up_numpy_type(numpy, "int16\0"),
            int32: look_up_numpy_type(numpy, "int32\0"),
            int64: look_up_numpy_type(numpy, "int64\0"),
            uint16: look_up_numpy_type(numpy, "uint16\0"),
            uint32: look_up_numpy_type(numpy, "uint32\0"),
            uint64: look_up_numpy_type(numpy, "uint64\0"),
            uint8: look_up_numpy_type(numpy, "uint8\0"),
            bool_: look_up_numpy_type(numpy, "bool_\0"),
        });
        Py_XDECREF(numpy);
        Box::new(Some(nonnull!(Box::<NumpyTypes>::into_raw(types))))
    }
}

macro_rules! pymod {
    ($module:literal) => {
        PyImport_ImportModule($module.as_ptr() as *const c_char)
    };
}

#[cold]
pub fn load_lise_types() -> Box<Option<NonNull<LiSETypes>>> {
    unsafe {
        let tblib = pymod!("tblib\0");
        let util_mod = pymod!("LiSE.util\0");
        let char_mod = pymod!("LiSE.character\0");
        let proxy_mod = pymod!("LiSE.proxy\0");
        let node_mod = pymod!("LiSE.node\0");
        let portal_mod = pymod!("LiSE.portal\0");

        let types = Box::new(LiSETypes {
            character: look_up_numpy_type(char_mod, "Character\0"),
            character_proxy: look_up_numpy_type(proxy_mod, "CharacterProxy\0"),
            thing: look_up_numpy_type(node_mod, "Thing\0"),
            thing_proxy: look_up_numpy_type(proxy_mod, "ThingProxy\0"),
            place: look_up_numpy_type(node_mod, "Place\0"),
            place_proxy: look_up_numpy_type(proxy_mod, "PlaceProxy\0"),
            portal: look_up_numpy_type(portal_mod, "Portal\0"),
            portal_proxy: look_up_numpy_type(proxy_mod, "PortalProxy\0"),
            final_rule: look_up_numpy_type(util_mod, "FinalRule\0"),
            traceback: PyObject_GetAttrString(tblib, "Traceback\0".as_ptr() as *const c_char),
        });

        Py_XDECREF(portal_mod);
        Py_XDECREF(node_mod);
        Py_XDECREF(proxy_mod);
        Py_XDECREF(char_mod);
        Py_XDECREF(util_mod);
        Py_XDECREF(tblib);
        Py_XDECREF(types.traceback);
        Box::new(Some(nonnull!(Box::<LiSETypes>::into_raw(types))))
    }
}

#[cold]
unsafe fn look_up_field_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("dataclasses\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "_FIELD\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
unsafe fn look_up_enum_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("enum\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "EnumMeta\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
unsafe fn look_up_uuid_type() -> *mut PyTypeObject {
    let uuid_mod = PyImport_ImportModule("uuid\0".as_ptr() as *const c_char);
    let uuid_mod_dict = PyObject_GenericGetDict(uuid_mod, null_mut());
    let uuid = PyMapping_GetItemString(uuid_mod_dict, "NAMESPACE_DNS\0".as_ptr() as *const c_char);
    let ptr = (*uuid).ob_type;
    Py_DECREF(uuid);
    Py_DECREF(uuid_mod_dict);
    Py_DECREF(uuid_mod);
    ptr
}

#[cold]
unsafe fn look_up_datetime_type() -> *mut PyTypeObject {
    let datetime_api = *PyDateTimeAPI();
    let datetime = (datetime_api.DateTime_FromDateAndTime)(
        1970,
        1,
        1,
        0,
        0,
        0,
        0,
        NONE,
        datetime_api.DateTimeType,
    );
    let ptr = (*datetime).ob_type;
    Py_DECREF(datetime);
    ptr
}

#[cold]
unsafe fn look_up_date_type() -> *mut PyTypeObject {
    let datetime_api = *PyDateTimeAPI();
    let date = (datetime_api.Date_FromDate)(1970, 1, 1, datetime_api.DateType);
    let ptr = (*date).ob_type;
    Py_DECREF(date);
    ptr
}

#[cold]
unsafe fn look_up_time_type() -> *mut PyTypeObject {
    let datetime_api = *PyDateTimeAPI();
    let time = (datetime_api.Time_FromTime)(0, 0, 0, 0, NONE, datetime_api.TimeType);
    let ptr = (*time).ob_type;
    Py_DECREF(time);
    ptr
}
