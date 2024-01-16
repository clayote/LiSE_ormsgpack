# SPDX-License-Identifier: (Apache-2.0 OR MIT)
import pytest

import lise_ormsgpack


def test_circular_dict():
    """
    packb() circular reference dict
    """
    obj = {}
    obj["obj"] = obj
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)


def test_circular_list():
    """
    packb() circular reference list
    """
    obj = []
    obj.append(obj)
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)


def test_circular_nested():
    """
    packb() circular reference nested dict, list
    """
    obj = {}
    obj["list"] = [{"obj": obj}]
    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(obj)
