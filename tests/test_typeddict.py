# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import lise_ormsgpack

try:
    from typing import TypedDict
except ImportError:
    from typing_extensions import TypedDict


def test_typeddict():
    """
    packb() TypedDict
    """

    class TypedDict1(TypedDict):
        a: str
        b: int

    obj = TypedDict1(a="a", b=1)

    assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(obj)) == {"a": "a", "b": 1}
