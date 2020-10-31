"""Provides IPMI PMBus related functions."""
from .models import PowerSupply


def process_pmbus_response(psu_list: list) -> list:
    """Obtain all power supplies.

    Args:
        psu_list: List of power supplies obtained from an XML response.

    Returns:
        list: Fully populated power supplies, complete with ID.
    """
    power_supplies = []
    psu_id = 0
    for item in psu_list:
        psu = process_pmbus_psu(item)
        psu.id = psu_id
        psu_id += 1
        power_supplies.append(psu)

    return power_supplies


def process_pmbus_psu(item: dict) -> PowerSupply:
    """Process a single power supply.

    Args:
        item: A single power supply obtained from an XML response.

    Returns:
        PowerSupply: Fully populated power supply, minus the ID.
    """
    psu = PowerSupply()
    psu.name = item["name"]
    psu.status = item["a_b_PS_Status_I2C"]
    psu.type = item["psType"]
    psu.input_voltage = int(item["acInVoltage"], 16)
    psu.input_current = int(item["acInCurrent"], 16) / 1000
    psu.input_power = int(item["acInPower"], 16)
    psu.output_voltage = int(item["dc12OutVoltage"], 16) / 10
    psu.output_current = int(item["dc12OutCurrent"], 16) / 1000
    psu.output_power = int(item["dcOutPower"], 16)
    psu.temp_1 = int(item["temp1"], 16)
    psu.temp_2 = int(item["temp2"], 16)
    psu.fan_1 = int(item["fan1"], 16)
    psu.fan_2 = int(item["fan2"], 16)

    # for key, value in item.items():
    #     if "temp" in key:
    #         sensor = Sensor()
    #         sensor.name = key
    #         sensor.reading = int(value, 16)
    #         sensor.type = SensorTypeEnum.TEMPERATURE
    #         sensor.unit = SensorUnitEnum.DEGREES_CELSIUS
    #         psu.sensors.append(sensor)
    #     elif "fan" in key:
    #         sensor = Sensor()
    #         sensor.name = key
    #         sensor.reading = int(value, 16)
    #         sensor.type = SensorTypeEnum.FAN
    #         sensor.unit = SensorUnitEnum.RPM
    #         psu.sensors.append(sensor)

    return psu
