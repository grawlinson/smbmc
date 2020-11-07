SMBMC
=====

An unofficial Python interface for obtaining metrics from Supermicro BMCs.

The following metrics are accessible:

- Sensor: Temperature, Fan, Voltage, etc.
- PMBus: Power Consumption, Fan, Temperature, etc.

**Note:** This library depends on the BMC web-interface being available.

Usage
-----

Setup
~~~~~

::

    # smbmc_example.py
    from smbmc import Client

    # initialise client with connection details
    c = Client(IPMI_SERVER, IPMI_USER, IPMI_PASS)

    # retrieve session token (usually lasts 30 minutes)
    # optional: the library maintains session tokens internally.
    c.login()


Sensor Metrics
~~~~~~~~~~~~~~

::

    # obtain sensor metrics
    sensors = c.get_sensor_metrics()

    for sensor in sensors:
        print(sensor.__dict__)

    # output (some removed for brevity)
    {'id': 0, 'name': 'System Temp', 'type': <SensorTypeEnum.TEMPERATURE: 1>, 'unit': <SensorUnitEnum.DEGREES_CELSIUS: 1>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 28.0, 'lnr': -9.0, 'lc': -7.0, 'lnc': -5.0, 'unc': 80.0, 'uc': 85.0, 'unr': 90.0}
    {'id': 1, 'name': '12VCC', 'type': <SensorTypeEnum.VOLTAGE: 2>, 'unit': <SensorUnitEnum.VOLTS: 4>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 12.192, 'lnr': 10.144, 'lc': 10.272, 'lnc': 10.784, 'unc': 12.96, 'uc': 13.28, 'unr': 13.408}
    {'id': 9, 'name': 'FAN1', 'type': <SensorTypeEnum.FAN: 4>, 'unit': <SensorUnitEnum.RPM: 18>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 3500.0, 'lnr': 400.0, 'lc': 600.0, 'lnc': 800.0, 'unc': 25300.0, 'uc': 25400.0, 'unr': 25500.0}
    {'id': 10, 'name': 'FAN2', 'type': <SensorTypeEnum.FAN: 4>, 'unit': <SensorUnitEnum.RPM: 18>, 'state': <SensorStateEnum.NOT_PRESENT: 2>, 'flags': None, 'reading': 0.0, 'lnr': 400.0, 'lc': 600.0, 'lnc': 800.0, 'unc': 25300.0, 'uc': 25400.0, 'unr': 25500.0}
    {'id': 19, 'name': 'SAS2 FTemp1', 'type': <SensorTypeEnum.TEMPERATURE: 1>, 'unit': <SensorUnitEnum.DEGREES_CELSIUS: 1>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 30.0, 'lnr': -9.0, 'lc': -7.0, 'lnc': -5.0, 'unc': 75.0, 'uc': 77.0, 'unr': 79.0}
    {'id': 27, 'name': 'PS2 Status', 'type': <SensorTypeEnum.POWER_SUPPLY: 8>, 'unit': <SensorUnitEnum.UNSPECIFIED: 0>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': <PowerSupplyFlag.PRESENCE_DETECTED: 1>, 'reading': 0, 'lnr': 0, 'lc': 0, 'lnc': 0, 'unc': 0, 'uc': 0, 'unr': 0}

PMBus Metrics
~~~~~~~~~~~~~

::

    # obtain pmbus metrics
    power_supplies = c.get_pmbus_metrics()

    for psu in power_supplies:
        print(psu.__dict__)

    # output
    {'id': 0, 'name': '', 'status': 'ff', 'type': '0', 'input_voltage': 0, 'input_current': 0.0, 'input_power': 0, 'output_voltage': 0.0, 'output_current': 0.0, 'output_power': 0, 'temp_1': 0, 'temp_2': 0, 'fan_1': 0, 'fan_2': 0}
    {'id': 1, 'name': 'FAKE_PSU_SERIAL', 'status': '1', 'type': '1', 'input_voltage': 236, 'input_current': 0.359, 'input_power': 85, 'output_voltage': 12.1, 'output_current': 5.75, 'output_power': 69, 'temp_1': 38, 'temp_2': 53, 'fan_1': 2894, 'fan_2': 3810}


All Metrics
~~~~~~~~~~~

::

    # obtain all metrics
    metrics = c.get_metrics()

    print(metrics)

    # output
    {'pmbus': [], 'sensor': []}


Contributing
------------

This library has been tested on a system with the following components:

- Chassis: SC846 (unknown revision; possibly 846BA-R920B)
- Motherboard: CSE-PTJBOD-CB3
- Power Supply: PWS-920P-SQ
- Backplane: BPN-SAS2-846EL1
- Power Distribution Board: PDB-PT846-2824

If there are any errors or additional functionality for other components, please file an issue with as *much* detail as you can!

Legal
-----

This library is not associated with Super Micro Computer, Inc.

Supermicro have released some `BMC/IPMI <https://www.supermicro.com/wftp/GPL/SMT/SDK_SMT_X9_317.tar.gz>`_ code under the GPL, which has been used as a reference. Therefore, this library is licensed as GPLv3.
