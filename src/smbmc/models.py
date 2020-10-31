"""Provides models."""
from enum import IntEnum
from enum import IntFlag


class SensorStateEnum(IntEnum):
    """Enumeration of sensor states."""

    UNSPECIFIED = 0
    PRESENT = 1
    NOT_PRESENT = 2


class SensorUnitEnum(IntEnum):
    """Enumeration of sensor units."""

    UNSPECIFIED = 0
    DEGREES_CELSIUS = 1
    VOLTS = 4
    AMPS = 5
    WATTS = 6
    RPM = 18


class SensorTypeEnum(IntEnum):
    """Enumeration of sensor types."""

    UNSPECIFIED = 0
    TEMPERATURE = 1
    VOLTAGE = 2
    FAN = 4
    POWER_SUPPLY = 8


class PowerSupplyFlag(IntFlag):
    """Flags for power supply specific events."""

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
        unit: Reading unit. e.g. Temperature - degrees Celsius.
        state: Sensor state.
        flags: Discrete sensors only; type specific flags.
        reading: Sensor reading.
        lnr: Lower non-recoverable indicator.
        lc: Lower critical indicator.
        lnc: Lower non-critical indicator.
        unc: Upper non-critical indicator.
        uc: Upper critical indicator.
        unr: Upper non-recoverable indicator.
    """

    def __init__(self):
        """Creates an instance of the Sensor class."""
        self.id = 0
        self.name = ""
        self.type = SensorTypeEnum.UNSPECIFIED
        self.unit = SensorUnitEnum.UNSPECIFIED
        self.state = SensorStateEnum.UNSPECIFIED
        self.flags = None
        # values
        self.reading = 0
        self.lnr = 0
        self.lc = 0
        self.lnc = 0
        self.unc = 0
        self.uc = 0
        self.unr = 0


class PowerSupply:
    """PowerSupply provides an interface to power supplies.

    Attributes:
        id: Psuedo-unique id.
        name: Serial number.
        status: Not 100% sure, but possibly status relayed via I2C.
        type: Not 100% sure on this either.
        input_voltage: Input voltage (V_AC).
        input_current: Input current (A).
        input_power: Input power (W).
        output_voltage: Output voltage (V_DC).
        output_current: Output current (A).
        output_power: Output power (W).
        temp_1: Temperature 1. Possibly intake (degrees Celsius).
        temp_2: Temperature 2. Possibly outlet (degrees Celsius).
        fan_1: Fan 1. Possibly intake (r.p.m.).
        fan_2: Fan 2. Possibly outlet (r.p.m.).
    """

    def __init__(self):
        """Creates an instance of the PowerSupply class."""
        # id : slot no. in server chassis
        # name : power supply serial number
        # status : most likely i2c status
        #  [1=i2c_enabled,0=i2c_disabled]
        # type : most likely power status
        #  [1=on,0=disconnected/off/standby]
        self.id = 0
        self.name = ""
        self.status = 0
        self.type = 0
        # input characteristics
        self.input_voltage = 0
        self.input_current = 0
        self.input_power = 0
        # output characteristics
        self.output_voltage = 0
        self.output_current = 0
        self.output_power = 0
        # sensors
        # self.sensors = []
        self.temp_1 = 0
        self.temp_2 = 0
        self.fan_1 = 0
        self.fan_2 = 0
