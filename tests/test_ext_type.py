from functools import partial

import msgpack
import pytest

import ormsgpack


def test_ext_type():
    tag = 1
    data = b"test"
    packed = ormsgpack.packb(ormsgpack.Ext(tag, data))
    assert packed == msgpack.packb(msgpack.ExtType(tag, data))

    unpacked = ormsgpack.unpackb(
        packed,
        ext_hook=lambda x, y: (x, y),
    )
    assert unpacked == (tag, data)

    unpacked = ormsgpack.unpackb(
        packed,
        ext_hook=lambda x, y: (x, y),
        option=ormsgpack.OPT_NON_STR_KEYS,
    )
    assert unpacked == (tag, data)

    with pytest.raises(ormsgpack.MsgpackDecodeError):
        ormsgpack.unpackb(packed)


def test_ext_type_tuple():

    orpacker = partial(ormsgpack.packb,
                       option=ormsgpack.OPT_NON_STR_KEYS)

    def corepack_handler(obj):
        if isinstance(obj, tuple):
            return msgpack.ExtType(0x00,
                                   corepacker(list(obj)))
        raise TypeError("unpackable")

    corepacker = partial(msgpack.packb, default=corepack_handler, strict_types=False)

    test_data_0 = {(0, 0): True, (0, 1): False, (1, 0): True, (1, 1): False}

    assert orpacker(test_data_0) == corepacker(test_data_0)

    data = {'physical': {(0, 0): {'pos': (0.5, 0.0), '_x': 0.0, '_y': 0.0, 'name': (0, 0)},
                         (0, 1): {'pos': (0.0, 0.8660254037844386), '_x': 0.0, '_y': 0.023902439024390244,
                                  'name': (0, 1)}}}

    assert orpacker(data) == corepacker(data)
