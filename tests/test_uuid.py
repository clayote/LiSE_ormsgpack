# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import uuid

import pytest

import lise_ormsgpack


def test_uuid_subclass():
    """
    UUID subclasses are not serialized
    """

    class AUUID(uuid.UUID):
        pass

    with pytest.raises(lise_ormsgpack.MsgpackEncodeError):
        lise_ormsgpack.packb(AUUID("{12345678-1234-5678-1234-567812345678}"))


def test_nil_uuid():
    assert (
            lise_ormsgpack.unpackb(
            lise_ormsgpack.packb(uuid.UUID("00000000-0000-0000-0000-000000000000"))
        )
            == "00000000-0000-0000-0000-000000000000"
    )


def test_all_ways_to_create_uuid_behave_equivalently():
    # Note that according to the docstring for the uuid.UUID class, all the
    # forms below are equivalent -- they end up with the same value for
    # `self.int`, which is all that really matters
    uuids = [
        uuid.UUID("{12345678-1234-5678-1234-567812345678}"),
        uuid.UUID("12345678123456781234567812345678"),
        uuid.UUID("urn:uuid:12345678-1234-5678-1234-567812345678"),
        uuid.UUID(bytes=b"\x12\x34\x56\x78" * 4),
        uuid.UUID(
            bytes_le=b"\x78\x56\x34\x12\x34\x12\x78\x56"
            + b"\x12\x34\x56\x78\x12\x34\x56\x78"
        ),
        uuid.UUID(fields=(0x12345678, 0x1234, 0x5678, 0x12, 0x34, 0x567812345678)),
        uuid.UUID(int=0x12345678123456781234567812345678),
    ]
    result = lise_ormsgpack.unpackb(lise_ormsgpack.packb(uuids))
    packed = [str(u) for u in uuids]
    assert packed == result


def test_serializes_correctly_with_leading_zeroes():
    instance = uuid.UUID(int=0x00345678123456781234567812345678)
    assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(instance)) == str(instance)


def test_all_uuid_creation_functions_create_serializable_uuids():
    uuids = (
        uuid.uuid1(),
        uuid.uuid3(uuid.NAMESPACE_DNS, "python.org"),
        uuid.uuid4(),
        uuid.uuid5(uuid.NAMESPACE_DNS, "python.org"),
    )
    for val in uuids:
        assert lise_ormsgpack.unpackb(lise_ormsgpack.packb(val)) == str(val)
