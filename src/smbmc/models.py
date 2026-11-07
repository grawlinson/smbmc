"""Provides models."""

from __future__ import annotations

from enum import IntEnum, IntFlag
from typing import Any


class _CodeEnum(IntEnum):
    """IntEnum that tolerates codes it does not know.

    Unknown codes become pseudo-members named ``UNKNOWN_<code>`` instead of
    raising :class:`ValueError`.
    """

    @classmethod
    def _missing_(cls, value: object) -> _CodeEnum | None:
        if not isinstance(value, int):
            return None
        member = int.__new__(cls, value)
        member._name_ = f"UNKNOWN_{value}"
        member._value_ = value
        return member


class SensorStateEnum(_CodeEnum):
    """Enumeration of sensor states.

    Possible Values:

    - Unspecified
    - Present
    - Not Present
    """

    UNSPECIFIED = 0
    PRESENT = 1
    NOT_PRESENT = 2


class SensorUnitEnum(_CodeEnum):
    """Enumeration of sensor units.

    Possible Values:

    - Unspecified
    - degrees Celsius (°C)
    - Volts (V)
    - Amps (A)
    - Watts (W)
    - revolutions per minute (rpm)

    Unknown units are represented as ``UNKNOWN_<code>`` pseudo-members.
    """

    UNSPECIFIED = 0
    DEGREES_CELSIUS = 1
    VOLTS = 4
    AMPS = 5
    WATTS = 6
    RPM = 18


class SensorTypeEnum(_CodeEnum):
    """Enumeration of sensor types.

    Possible Values:

    - Unspecified
    - Temperature
    - Voltage
    - Fan
    - Power Supply

    Unknown types are represented as ``UNKNOWN_<code>`` pseudo-members.
    """

    UNSPECIFIED = 0
    TEMPERATURE = 1
    VOLTAGE = 2
    FAN = 4
    POWER_SUPPLY = 8


class PowerSupplyFlag(IntFlag):
    """Bitflag for power supply specific events.

    Flags:

    - Unspecified
    - Presence Detected
    - Failure
    - Predictive Failure
    - Source Input Lost
    - Source Input Out of Range
    - Source Input Detected (Out of Range)
    - Configuration Error
    - Standby
    """

    UNSPECIFIED = 0
    PRESENCE_DETECTED = 1
    FAILURE = 2
    PREDICTIVE_FAILURE = 4
    SOURCE_INPUT_LOST = 8
    SOURCE_INPUT_OUT_OF_RANGE = 16
    SOURCE_INPUT_DETECTED_OUT_OF_RANGE = 32
    CONFIGURATION_ERROR = 64
    STANDBY = 128


class Sensor:
    """Sensor provides an interface to sensors.

    Attributes:
        id: Psuedo-unique id.
        name: Sensor name.
        type: Sensor type.
        unit: Reading unit.
        state: Sensor state.
        flags: Discrete power supply sensors only; type specific flags.
        reading: Sensor reading.
        lnr: Lower non-recoverable threshold.
        lc: Lower critical threshold.
        lnc: Lower non-critical threshold.
        unc: Upper non-critical threshold.
        uc: Upper critical threshold.
        unr: Upper non-recoverable threshold.
    """

    def __init__(self) -> None:
        """Creates an instance of the Sensor class."""
        self.id = 0
        self.name = ""
        self.type = SensorTypeEnum.UNSPECIFIED
        self.unit = SensorUnitEnum.UNSPECIFIED
        self.state = SensorStateEnum.UNSPECIFIED
        self.flags: PowerSupplyFlag | None = None
        # values
        self.reading = 0.0
        self.lnr = 0.0
        self.lc = 0.0
        self.lnc = 0.0
        self.unc = 0.0
        self.uc = 0.0
        self.unr = 0.0

    @classmethod
    def _from_raw(cls, raw: dict[str, Any]) -> Sensor:
        """Build a Sensor from a dict returned by the native module."""
        sensor = cls()
        sensor.__dict__.update(raw)
        sensor.type = SensorTypeEnum(raw["type"])
        sensor.unit = SensorUnitEnum(raw["unit"])
        sensor.state = SensorStateEnum(raw["state"])
        if raw["flags"] is not None:
            sensor.flags = PowerSupplyFlag(raw["flags"])
        return sensor

    def __repr__(self) -> str:
        """Debug representation."""
        return f"Sensor({self.__dict__!r})"


class PowerSupply:
    """PowerSupply provides an interface to power supplies.

    Attributes:
        id: Psuedo-unique id.
        name: Serial number.
        status: Not 100% sure, but possibly status relayed via I2C (hex string).
        type: Not 100% sure on this either (hex string).
        input_voltage: Input voltage (V_AC).
        input_current: Input current (A).
        input_power: Input power (W).
        output_voltage: Output voltage (V_DC).
        output_current: Output current (A).
        output_power: Output power (W).
        temp_1: Temperature 1. Possibly intake (°C).
        temp_2: Temperature 2. Possibly outlet (°C).
        fan_1: Fan 1. Possibly intake (rpm).
        fan_2: Fan 2. Possibly outlet (rpm).
    """

    def __init__(self) -> None:
        """Creates an instance of the PowerSupply class."""
        # id : slot no. in server chassis
        # name : power supply serial number
        # status : most likely i2c status
        #  [1=i2c_enabled,ff=absent]
        # type : most likely power status
        #  [1=on,0=disconnected/off/standby]
        self.id = 0
        self.name = ""
        self.status = "0"
        self.type = "0"
        # input characteristics
        self.input_voltage = 0
        self.input_current = 0.0
        self.input_power = 0
        # output characteristics
        self.output_voltage = 0.0
        self.output_current = 0.0
        self.output_power = 0
        # sensors
        self.temp_1 = 0
        self.temp_2 = 0
        self.fan_1 = 0
        self.fan_2 = 0

    @classmethod
    def _from_raw(cls, raw: dict[str, Any]) -> PowerSupply:
        """Build a PowerSupply from a dict returned by the native module."""
        psu = cls()
        psu.__dict__.update(raw)
        # Kept as lowercase hex strings for compatibility with 0.3.x.
        psu.status = format(raw["status"], "x")
        psu.type = format(raw["type"], "x")
        return psu

    def __repr__(self) -> str:
        """Debug representation."""
        return f"PowerSupply({self.__dict__!r})"
