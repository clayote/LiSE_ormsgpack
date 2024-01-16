# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import collections

import msgpack
import pytest

import lise_ormsgpack


class SubStr(str):
    pass


class SubInt(int):
    pass


class SubDict(dict):
    pass


class SubList(list):
    pass


class SubFloat(float):
    pass


class SubTuple(tuple):
    pass


def test_subclass_str():
    assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(SubStr("zxc"))) == "zxc"


def test_subclass_str_invalid():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubStr("\ud800"))


def test_subclass_int():
    assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(SubInt(1))) == 1


def test_subclass_int_64():
    for val in (9223372036854775807, -9223372036854775807):
        assert lise_ormsgpack.packb(SubInt(val)) == msgpack.packb(val)


def test_subclass_dict():
    assert lise_ormsgpack.packb(SubDict({"a": "b"})) == msgpack.packb({"a": "b"})


def test_subclass_list():
    assert lise_ormsgpack.packb(SubList(["a", "b"])) == msgpack.packb(["a", "b"])

    ref = [True] * 512
    assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(SubList(ref))) == ref


def test_subclass_float():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubFloat(1.1))


def test_subclass_tuple():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubTuple((1, 2)))


def test_namedtuple():
    Point = collections.namedtuple("Point", ["x", "y"])
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(Point(1, 2))


def test_subclass_circular_dict():
    obj = SubDict({})
    obj["obj"] = obj
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)


def test_subclass_circular_list():
    obj = SubList([])
    obj.append(obj)
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)


def test_subclass_circular_nested():
    obj = SubDict({})
    obj["list"] = SubList([{"obj": obj}])
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)


def test_subclass_str_opt():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubStr("zxc"), option=lise_ormsgpack.OPT_PASSTHROUGH_SUBCLASS)


def test_subclass_int_opt():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubInt(1), option=lise_ormsgpack.OPT_PASSTHROUGH_SUBCLASS)


def test_subclass_dict_opt():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubDict({"a": "b"}), option=lise_ormsgpack.OPT_PASSTHROUGH_SUBCLASS)


def test_subclass_list_opt():
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(SubList(["a", "b"]), option=lise_ormsgpack.OPT_PASSTHROUGH_SUBCLASS)
