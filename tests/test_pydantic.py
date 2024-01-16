from typing import Optional

from pydantic import BaseModel

import lise_ormsgpack


class Model1(BaseModel):
    hi: str
    number: int
    sub: Optional[int]


class Model2(BaseModel):
    bye: str
    previous: Model1


def test_basemodel():
    """
    packb() pydantic basemodel
    """
    obj = Model1(hi="a", number=1, sub=None)
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_SERIALIZE_PYDANTIC)
    assert lise_ormsgpack.unpackb(packed) == {"hi": "a", "number": 1, "sub": None}


def test_recursive_basemodel():
    """
    packb() pydantic basemodel with another basemodel as attribute
    """
    obj = Model1(hi="a", number=1, sub=None)
    obj2 = Model2(previous=obj, bye="lala")
    packed = lise_ormsgpack.packb(obj2, option=lise_ormsgpack.OPT_SERIALIZE_PYDANTIC)
    assert lise_ormsgpack.unpackb(packed) == {
        "bye": "lala",
        "previous": {"hi": "a", "number": 1, "sub": None},
    }
