"""Unit tests for models."""

import pytest

from smbmc import (
    PowerSupplyFlag,
    Sensor,
    SensorStateEnum,
    SensorTypeEnum,
    SensorUnitEnum,
)


@pytest.mark.parametrize("enum", [SensorStateEnum, SensorTypeEnum, SensorUnitEnum])
def test_unknown_codes(enum):
    """Unknown codes become pseudo-members instead of raising ValueError."""
    member = enum(200)
    assert member == 200
    assert member.name == "UNKNOWN_200"
    assert isinstance(member, enum)


def test_non_int_still_rejected():
    """Only integers are accepted as unknown codes."""
    with pytest.raises(ValueError, match="not a valid"):
        SensorTypeEnum("temperature")


def test_flags_keep_unknown_bits():
    """IntFlag keeps any bits it does not name."""
    assert int(PowerSupplyFlag(0b1_0000_0001)) == 0b1_0000_0001


def test_sensor_defaults():
    """A blank Sensor has sensible defaults."""
    sensor = Sensor()
    assert sensor.type is SensorTypeEnum.UNSPECIFIED
    assert sensor.flags is None
    assert repr(sensor).startswith("Sensor(")
