# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import dataclasses
import datetime
import uuid

import msgpack
import pytest
import pytz

import lise_ormsgpack


class SubStr(str):
    pass


def test_dict_keys_substr():
    assert lise_ormsgpack.packb(
        {SubStr("aaa"): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({"aaa": True})


def test_dict_keys_substr_passthrough():
    """
    OPT_PASSTHROUGH_SUBCLASS does not affect OPT_NON_STR_KEYS
    """
    assert lise_ormsgpack.packb(
        {SubStr("aaa"): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS | lise_ormsgpack.OPT_PASSTHROUGH_SUBCLASS,
    ) == msgpack.packb({"aaa": True})


def test_dict_keys_int_range_valid_i64():
    """
    OPT_NON_STR_KEYS has a i64 range for int, valid
    """
    assert lise_ormsgpack.packb(
        {9223372036854775807: True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({9223372036854775807: True})
    assert lise_ormsgpack.packb(
        {-9223372036854775807: True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({-9223372036854775807: True})
    assert lise_ormsgpack.packb(
        {9223372036854775809: True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({9223372036854775809: True})


def test_dict_keys_int_range_valid_u64():
    """
    OPT_NON_STR_KEYS has a u64 range for int, valid
    """
    obj = {0: True}
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    assert obj == lise_ormsgpack.unpackb(packed, option=lise_ormsgpack.OPT_NON_STR_KEYS)

    assert lise_ormsgpack.packb(
        {18446744073709551615: True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({18446744073709551615: True})


def test_dict_keys_int_range_invalid():
    """
    OPT_NON_STR_KEYS has a range of i64::MIN to u64::MAX
    """
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({-9223372036854775809: True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({18446744073709551616: True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_float():
    obj = {1.1: True, 2.2: False}
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    assert obj == lise_ormsgpack.unpackb(packed, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_inf():
    assert lise_ormsgpack.packb(
        {float("Infinity"): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({float("Infinity"): True})
    assert lise_ormsgpack.packb(
        {float("-Infinity"): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({float("-Infinity"): True})


def test_dict_keys_nan():
    assert lise_ormsgpack.packb(
        {float("NaN"): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({float("NaN"): True})


def test_dict_keys_bool():
    obj = {True: True, False: False}
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_NON_STR_KEYS)
    assert packed == msgpack.packb(obj)
    assert lise_ormsgpack.unpackb(packed, option=lise_ormsgpack.OPT_NON_STR_KEYS) == obj


def test_dict_keys_datetime():
    assert lise_ormsgpack.packb(
        {datetime.datetime(2000, 1, 1, 2, 3, 4, 123): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({"2000-01-01T02:03:04.000123": True})


def test_dict_keys_datetime_opt():
    assert lise_ormsgpack.packb(
        {datetime.datetime(2000, 1, 1, 2, 3, 4, 123): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS
               | lise_ormsgpack.OPT_OMIT_MICROSECONDS
               | lise_ormsgpack.OPT_NAIVE_UTC
               | lise_ormsgpack.OPT_UTC_Z,
    ) == msgpack.packb({"2000-01-01T02:03:04Z": True})


def test_dict_keys_datetime_passthrough():
    """
    OPT_PASSTHROUGH_DATETIME does not affect OPT_NON_STR_KEYS
    """
    assert lise_ormsgpack.packb(
        {datetime.datetime(2000, 1, 1, 2, 3, 4, 123): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS | lise_ormsgpack.OPT_PASSTHROUGH_DATETIME,
    ) == msgpack.packb({"2000-01-01T02:03:04.000123": True})


def test_dict_keys_uuid():
    """
    OPT_NON_STR_KEYS always serializes UUID as keys
    """
    assert lise_ormsgpack.packb(
        {uuid.UUID("7202d115-7ff3-4c81-a7c1-2a1f067b1ece"): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({"7202d115-7ff3-4c81-a7c1-2a1f067b1ece": True})


def test_dict_keys_date():
    assert lise_ormsgpack.packb(
        {datetime.date(1970, 1, 1): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({"1970-01-01": True})


def test_dict_keys_time():
    assert lise_ormsgpack.packb(
        {datetime.time(12, 15, 59, 111): True},
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    ) == msgpack.packb({"12:15:59.000111": True})


def test_dict_non_str_and_sort_keys():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(
            {
                datetime.date(1970, 1, 3): 3,
                datetime.date(1970, 1, 5): 2,
                "other": 1,
            },
            option=lise_ormsgpack.OPT_NON_STR_KEYS | lise_ormsgpack.OPT_SORT_KEYS,
        )


def test_dict_keys_time_err():
    """
    OPT_NON_STR_KEYS propagates errors in types
    """
    val = datetime.time(12, 15, 59, 111, tzinfo=pytz.timezone("Asia/Shanghai"))
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({val: True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_str():
    assert lise_ormsgpack.packb(
        {"1": True}, option=lise_ormsgpack.OPT_NON_STR_KEYS
    ) == msgpack.packb({"1": True})


def test_dict_keys_type():
    class Obj:
        a: str

    val = Obj()
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({val: True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_dataclass_hash():
    @dataclasses.dataclass
    class Dataclass:
        a: str

        def __hash__(self):
            return 1

    obj = {Dataclass("a"): True}
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_tuple():
    obj = {(): True}
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_NON_STR_KEYS)
    with pytest.raises(lise_ormsgpack.MsgpackDecodeError):
        lise_ormsgpack.unpackb(packed)
    assert (
            lise_ormsgpack.unpackb(
            packed,
            option=lise_ormsgpack.OPT_NON_STR_KEYS,
        )
            == obj
    )


def test_dict_keys_unknown():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({frozenset(): True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_no_str_call():
    class Obj:
        a: str

        def __str__(self):
            return "Obj"

    val = Obj()
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb({val: True}, option=lise_ormsgpack.OPT_NON_STR_KEYS)


def test_dict_keys_bytes():
    data = {b"test": b"lala"}
    assert (
            lise_ormsgpack.unpackb(
            lise_ormsgpack.packb(data, option=lise_ormsgpack.OPT_NON_STR_KEYS),
            option=lise_ormsgpack.OPT_NON_STR_KEYS,
        )
            == data
    )
