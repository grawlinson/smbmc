"""Unit tests for IPMI sensor functions."""
import pytest

from smbmc.ipmi_sensor import get_sensor_state
from smbmc.ipmi_sensor import is_analog_data_format
from smbmc.ipmi_sensor import is_threshold_sensor
from smbmc.ipmi_sensor import perform_linearisation
from smbmc.ipmi_sensor import process_discrete_sensor
from smbmc.ipmi_sensor import process_sensor_response
from smbmc.ipmi_sensor import reading_conversion
from smbmc.models import Sensor
from smbmc.models import SensorStateEnum
from smbmc.util import extract_xml_attr


@pytest.mark.parametrize(
    "data,m,b,rb,expected_result",
    [
        ("84", "6400", "0000", "d0", 13.200000000000001),
        ("83", "6400", "0000", "d0", 13.1),
        ("7f", "6400", "0000", "d0", 12.700000000000001),
        ("6a", "6400", "0000", "d0", 10.6),
        ("65", "6400", "0000", "d0", 10.1),
        ("64", "6400", "0000", "d0", 10.0),
    ],
)
def test_reading_conversion(data, m, b, rb, expected_result):
    """Ensure sensor readings are converted properly.

    Args:
        data: Sensor reading.
        m: Multiplier value.
        b: Offset value.
        rb: RB Exponent value.
        expected_result: Floating point result.
    """
    assert reading_conversion(data, m, b, rb) == expected_result


@pytest.mark.parametrize(
    "er_type,expected_result",
    [
        ("0x01", True),
        ("1", True),
        ("FF", False),
        ("2", False),
        ("0x02", False),
    ],
)
def test_is_threshold_sensor(er_type, expected_result):
    """Ensure sensor type is correctly guessed.

    Args:
        er_type: Unmodified ERTYPE value.
        expected_result: Expected guess.
    """
    assert is_threshold_sensor(er_type) is expected_result


@pytest.mark.parametrize(
    "unit_type_1,expected_result",
    [
        ("0x80", True),
        ("91", True),
        ("0xC0", False),
        ("C0", False),
        ("0x00", False),
        ("00", False),
    ],
)
def test_is_analog_data_format(unit_type_1, expected_result):
    """Ensure analog data format is correctly guessed.

    Args:
        unit_type_1: Unmodified UNIT1 value.
        expected_result: Expected guess.
    """
    assert is_analog_data_format(unit_type_1) is expected_result


@pytest.mark.parametrize(
    "option,expected_result",
    [
        ("0x40", SensorStateEnum.PRESENT),
        ("0x80", SensorStateEnum.NOT_PRESENT),
    ],
)
def test_get_sensor_state(option, expected_result):
    """Ensure correct sensor state is returned.

    Args:
        option: Unmodified OPTION value.
        expected_result: Expected sensor state.
    """
    assert get_sensor_state(option) is expected_result


@pytest.mark.parametrize(
    "method,reading,expected_result",
    [
        ("0", 5.0000001, 5.0),
        ("0", 5.0, 5.0),
        ("0", 5.7, 5.7),
        ("0", 5.6000000000000005, 5.6),
        ("0", 13.200000000000001, 13.2),
        ("0", 13.1, 13.1),
        ("0", 12.700000000000001, 12.7),
        ("0", 0, 0),
    ],
)
def test_perform_linearisation(method, reading, expected_result):
    """Ensure linearisation is performed correctly.

    Args:
        method: Currently, only linear formula is implemented.
        reading: Pre-linearisation reading.
        expected_result: Expected result.
    """
    assert perform_linearisation(method, reading) == expected_result


def test_perform_linearisation_error():
    """Ensure unimplemented functionality raises an error."""
    with pytest.raises(NotImplementedError):
        assert perform_linearisation("02", "02")


def test_process_sensor_response():
    """Ensure all items returned are Sensor instances."""
    xml_file = "ipmi_response_sensors"
    selector = ".//SENSOR"
    xml_string = open(f"tests/unit/{xml_file}.xml").read()
    sensor_list = extract_xml_attr(xml_string, selector)
    sensors = process_sensor_response(sensor_list)

    assert len(sensors) == 28

    for sensor in sensors:
        assert isinstance(sensor, Sensor)


def test_process_threshold_sensor_error():
    """Ensure unimplemented sensor raises an error."""
    item = {}
    item["STYPE"] = "01"
    item["READING"] = "010100"
    item["OPTION"] = "c0"

    with pytest.raises(NotImplementedError):
        assert process_discrete_sensor(item)


def test_process_threshold_sensor_not_present():
    """Test result if discrete sensor is not present."""
    item = {}
    item["STYPE"] = "01"
    item["READING"] = "010100"
    item["OPTION"] = "00"

    assert process_discrete_sensor(item) is None
