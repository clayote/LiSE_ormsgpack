# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import lise_ormsgpack


def test_sort_keys():
    obj = {"b": 1, "c": 2, "a": 3, "ä": 4, "A": 5}
    packed = lise_ormsgpack.packb(obj, option=lise_ormsgpack.OPT_SORT_KEYS)
    unpacked = lise_ormsgpack.unpackb(packed)
    assert list(unpacked.keys()) == sorted(obj.keys())
