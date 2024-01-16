import msgpack
import pytest

import lise_ormsgpack


def test_ext_type():
    tag = 1
    data = b"test"
    packed = lise_ormsgpack.packb(lise_ormsgpack.Ext(tag, data))
    assert packed == msgpack.packb(msgpack.ExtType(tag, data))

    unpacked = lise_ormsgpack.unpackb(
        packed,
        ext_hook=lambda x, y: (x, y),
    )
    assert unpacked == (tag, data)

    unpacked = lise_ormsgpack.unpackb(
        packed,
        ext_hook=lambda x, y: (x, y),
        option=lise_ormsgpack.OPT_NON_STR_KEYS,
    )
    assert unpacked == (tag, data)

    with pytest.raises(lise_ormsgpack.MsgpackDecodeError):
        lise_ormsgpack.unpackb(packed)
