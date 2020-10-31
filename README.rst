SMBMC
=====

An unofficial Python interface for obtaining metrics from Supermicro BMCs.

The following metrics are accessible:

- Sensor: Temperature, Fan, Voltage, etc.
- PMBus: Power Consumption, Fan, Temperature, etc.

**Note:** This library depends on the BMC web-interface being available.

Usage
-----

::

    # smbmc_example.py
    from smbmc import Client

    # initialise client with connection details
    c = Client(IPMI_SERVER, IPMI_USER, IPMI_PASS)

    # retrieve session token
    c.login()

    # obtain sensor metrics
    sensors = c.get_sensor_metrics()

    # and pmbus metrics
    power_supplies = c.get_pmbus_metrics()

    # or, retrieve all known metrics
    metrics = c.get_metrics()

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
