"""Unit tests for sensor parsing (performed by the native extension)."""

import json
from pathlib import Path

import pytest

from smbmc import (
    PowerSupplyFlag,
    ResponseError,
    Sensor,
    SensorStateEnum,
    SensorTypeEnum,
    SensorUnitEnum,
    UnsupportedError,
    parse_sensors,
)

FIXTURES = Path(__file__).parents[2] / "crates" / "smbmc" / "tests" / "fixtures"


def sensor_xml(**attrs):
    """Build a single-sensor SENSOR_INFO response."""
    defaults = {
        "NAME": "Test",
        "READING": "19c000",
        "OPTION": "c0",
        "UNR": "5a",
        "UC": "55",
        "UNC": "50",
        "LNC": "fb",
        "LC": "f9",
        "LNR": "f7",
        "STYPE": "01",
        "ERTYPE": "01",
        "UNIT1": "80",
        "UNIT": "01",
        "L": "00",
        "M": "0100",
        "B": "0000",
        "RB": "00",
    }
    defaults.update(attrs)
    attributes = " ".join(f'{k}="{v}"' for k, v in defaults.items())
    return f"<IPMI><SENSOR_INFO><SENSOR {attributes}/></SENSOR_INFO></IPMI>"


def test_process_sensor_response():
    """All 28 sensors are parsed into Sensor instances."""
    sensors = parse_sensors((FIXTURES / "ipmi_response_sensors.xml").read_text())

    assert len(sensors) == 28
    for i, sensor in enumerate(sensors):
        assert isinstance(sensor, Sensor)
        assert sensor.id == i


def test_matches_python_0_3_0():
    """Results are identical to the pure-Python 0.3.0 implementation."""
    sensors = parse_sensors((FIXTURES / "ipmi_response_sensors.xml").read_text())
    golden = json.loads((FIXTURES / "golden_sensors_v0.3.0.json").read_text())

    assert [s.__dict__ for s in sensors] == golden


def test_threshold_sensor():
    """Two's complement thresholds and enum types."""
    (sensor,) = parse_sensors(sensor_xml())

    assert sensor.type is SensorTypeEnum.TEMPERATURE
    assert sensor.unit is SensorUnitEnum.DEGREES_CELSIUS
    assert sensor.state is SensorStateEnum.PRESENT
    assert sensor.flags is None
    assert (sensor.reading, sensor.lnr, sensor.unr) == (25.0, -9.0, 90.0)


def test_rounding():
    """Readings are rounded, not truncated (0.3.0 gave 4.054 here)."""
    (sensor,) = parse_sensors(
        sensor_xml(READING="96c000", UNIT1="00", UNIT="04", M="1b00", B="0500", RB="d0")
    )
    assert sensor.reading == 4.055


def test_power_supply_flags():
    """Discrete power supply sensors expose their state bits."""
    (sensor,) = parse_sensors(sensor_xml(STYPE="08", ERTYPE="6f", READING="010300"))

    assert sensor.type is SensorTypeEnum.POWER_SUPPLY
    assert sensor.flags == PowerSupplyFlag.PRESENCE_DETECTED | PowerSupplyFlag.FAILURE


def test_discrete_sensor_not_present():
    """A missing discrete sensor no longer crashes the whole response."""
    (sensor,) = parse_sensors(sensor_xml(ERTYPE="6f", OPTION="00"))

    assert sensor.state is SensorStateEnum.NOT_PRESENT
    assert sensor.flags is None


def test_discrete_sensor_unsupported_type():
    """Discrete sensors other than power supplies have no flags (yet)."""
    (sensor,) = parse_sensors(sensor_xml(ERTYPE="6f", STYPE="07"))

    assert sensor.state is SensorStateEnum.PRESENT
    assert sensor.flags is None
    assert sensor.type == 7
    assert sensor.type.name == "UNKNOWN_7"


def test_unknown_unit():
    """Unknown units are kept as UNKNOWN_<code> pseudo-members."""
    (sensor,) = parse_sensors(sensor_xml(UNIT="13"))

    assert sensor.unit == 0x13
    assert isinstance(sensor.unit, SensorUnitEnum)


def test_unsupported_linearisation():
    """Unimplemented linearisation formulas raise UnsupportedError."""
    with pytest.raises(UnsupportedError) as exc_info:
        parse_sensors(sensor_xml(L="02"))
    assert isinstance(exc_info.value, NotImplementedError)


@pytest.mark.parametrize(
    ("xml", "message"),
    [
        ("<IPMI>", "malformed"),
        ("<html/>", "expected <IPMI>"),
        ("<IPMI><SENSOR NAME='x'/></IPMI>", "ERTYPE"),
        (sensor_xml(M="zz"), "hexadecimal"),
    ],
)
def test_bad_responses(xml, message):
    """Unparseable responses raise ResponseError."""
    with pytest.raises(ResponseError, match=message):
        parse_sensors(xml)
