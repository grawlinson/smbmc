"""Provides IPMI sensor related functions."""
from enum import auto
from enum import IntEnum

from .models import PowerSupplyFlag
from .models import Sensor
from .models import SensorStateEnum
from .models import SensorTypeEnum
from .models import SensorUnitEnum
from .util import hex_signed_int
from .util import signed_int
from .util import ten_bit_str

SENSOR_READING_SCALE = 1000


class LinearisationEnum(IntEnum):
    """Enumeration of linearisation formulas."""

    LINEAR = 0
    LN = auto()
    LOG_10 = auto()
    LOG_2 = auto()
    EULER = auto()
    EXP_10 = auto()
    EXP_2 = auto()
    ONE_DIV_X = auto()
    SQR = auto()
    CUBE = auto()
    ONE_DIV_CUBE = auto()


def reading_conversion(data: str, m: str, b: str, rb: str) -> float:
    """Performs Sensor Reading Conversion Formula.

    TODO: This is the pre-linearisation formula, so look
    into the spec some more.

    Extracted from IPMI 2.0 specification, section 36.3.

    Args:
        data: Data reading.
        m: Multiplier value.
        b: Offset value.
        rb: RB Exponent value.

    Returns:
        float: Pre-linearisation reading.
    """
    from math import pow

    # Extracted from 43.1 - SDR Type 01h, bytes 25, 27, 30.
    m_raw = ten_bit_str(m)
    b_raw = ten_bit_str(b)
    km_raw = int(rb, 16) >> 4
    kb_raw = int(rb, 16) & 0x0F

    m_data = signed_int(m_raw, 10)
    b_data = signed_int(b_raw, 10)
    km_data = signed_int(km_raw, 4)
    kb_data = signed_int(kb_raw, 4)

    sensor_data = (m_data * int(data, 16) + b_data * pow(10, kb_data)) * pow(
        10, km_data
    )

    return float(sensor_data)


def is_threshold_sensor(er_type: str) -> bool:
    """Detects whether sensor is threshold or discrete.

    Extracted from the raw Event Reading Type field.

    Byte 14 - Event/Reading Type Code.

    Referenced from chapter 41.

    Args:
        er_type: Event Reading Type in raw form.

    Returns:
        bool: True if is a threshold sensor.
    """
    return int(er_type, 16) == 1


def is_analog_data_format(unit_type_1: str) -> bool:
    """Detects whether sensor reading is in analog data format.

    Args:
        unit_type_1: No clue.

    Returns:
        bool: True if analog data format.
    """
    return (int(unit_type_1, 16) >> 6) == 2


def get_sensor_state(option: str) -> SensorStateEnum:
    """Extract sensor state from OPTION byte.

    ... hopefully! Actually, probably a bad guess.

    Args:
        option: Raw OPTION byte from IPMI response.

    Returns:
        SensorStateEnum: State of the sensor.
    """
    if int(option, 16) & 0x40:
        return SensorStateEnum.PRESENT
    else:
        return SensorStateEnum.NOT_PRESENT


def perform_linearisation(method: str, reading: str) -> int:
    """Perform linearisation based on a method and given reading.

    Args:
        method: Linearisation method specified by IPMI response.
        reading: Reading obtained from IPMI response.

    Returns:
        int: Linearised sensor reading.

    Raises:
        NotImplementedError: Raised when linearisation methods have
            not been implemented.
    """
    i_method = int(method, 16)

    if i_method == LinearisationEnum.LINEAR:
        return int((reading * SENSOR_READING_SCALE)) / SENSOR_READING_SCALE
    else:
        raise NotImplementedError


def process_threshold_sensor(item: dict) -> Sensor:
    """Process a threshold sensor.

    Args:
        item: Dict representing a sensor, obtained from the IPMI response.

    Returns:
        Sensor: Fully populated sensor.
    """
    # values
    values = {}
    values["reading"] = item["READING"][:2]
    values["unr"] = item["UNR"]
    values["uc"] = item["UC"]
    values["unc"] = item["UNC"]
    values["lnc"] = item["LNC"]
    values["lc"] = item["LC"]
    values["lnr"] = item["LNR"]

    # linearisation variables
    multiplier = item["M"]
    offset = item["B"]
    rb_exponent = item["RB"]
    l_method = item["L"]

    # analog data conversion
    if is_analog_data_format(item["UNIT1"]):
        for key, value in values.items():
            values[key] = hex_signed_int(value)

    # perform linearisation of readings
    for key, value in values.items():
        values[key] = perform_linearisation(
            l_method, reading_conversion(value, multiplier, offset, rb_exponent)
        )

    # add a sensor and we've got a stew goin'!
    sensor = Sensor()
    sensor.name = item["NAME"]
    sensor.type = SensorTypeEnum(int(item["STYPE"], 16))
    sensor.unit = SensorUnitEnum(int(item["UNIT"], 16))
    sensor.state = get_sensor_state(item["OPTION"])
    sensor.reading = values["reading"]
    sensor.unc = values["unc"]
    sensor.uc = values["uc"]
    sensor.unr = values["unr"]
    sensor.lc = values["lc"]
    sensor.lnc = values["lnc"]
    sensor.lnr = values["lnr"]

    return sensor


def process_discrete_sensor(item: dict) -> Sensor:
    """Process a discrete sensor.

    TODO: finish this function.

    Args:
        item: Dict representing a sensor, obtained from the IPMI response.

    Returns:
        Sensor: Fully populated sensor.

    Raises:
        NotImplementedError: Raised when sensor type has not been implemented.
    """
    type = int(item["STYPE"], 16)
    reading = item["READING"]
    # raw_reading = reading[:2]
    # option = int(item["OPTION"], 16)
    sensor_d = int(reading[2:4], 16)
    # sensor_dmsb = int(reading[4:6], 16)
    # print(f"R:{reading} RR:{raw_reading} OPT:{option}")
    # print(f"SD:{sensor_d} S_DMSB:{sensor_dmsb}")
    sensor_state = get_sensor_state(item["OPTION"])

    # TODO: add edge-cases from utils.js
    if sensor_state is not SensorStateEnum.NOT_PRESENT:
        if type == SensorTypeEnum.POWER_SUPPLY:
            psu = Sensor()
            psu.name = item["NAME"]
            psu.type = SensorTypeEnum(type)
            psu.flags = PowerSupplyFlag(sensor_d)
            psu.state = sensor_state
            return psu
        else:
            raise NotImplementedError

    # utils.js: ShowDiscStateAPI( Sensor_Type, sensor_d )
    # servh_sensor: ProcDiscreteSensor(node,Idx)


def process_sensor_response(sensor_list: list) -> list:
    """Obtain all sensors.

    Args:
        sensor_list: List of sensors obtained from an XML response.

    Returns:
        list: Fully populated sensors.
    """
    sensors = []
    sensor_id = 0
    for item in sensor_list:
        sensor = process_sensor(item)
        sensor.id = sensor_id
        sensors.append(sensor)

        sensor_id += 1

    return sensors


def process_sensor(item: dict) -> Sensor:
    """Process a single sensor.

    Args:
        item: A single sensor obtained from an XML response.

    Returns:
        Sensor: Fully populated sensor.
    """
    if is_threshold_sensor(item["ERTYPE"]):
        sensor = process_threshold_sensor(item)
    else:
        sensor = process_discrete_sensor(item)

    return sensor
