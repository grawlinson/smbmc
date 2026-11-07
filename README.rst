SMBMC
=====

An unofficial interface for obtaining metrics from Supermicro BMCs, for Python
and Rust.

The following metrics are accessible:

- Sensor: Temperature, Fan, Voltage, etc.
- PMBus: Power Consumption, Fan, Temperature, etc.

**Note:** This library depends on the BMC web-interface being available.

Since 0.4.0 the library is written in Rust. The Python package (``smbmc`` on
PyPI) is a thin layer over the Rust crate (``smbmc`` on crates.io) and keeps the
0.3 API; see `Changes in 0.4.0`_.

Installation
------------

::

    # Python (3.10+); prebuilt wheels for Linux, macOS and Windows
    pip install smbmc

    # Rust
    cargo add smbmc                       # async client (Tokio)
    cargo add smbmc --features blocking   # plus the blocking client

Python Usage
------------

Setup
~~~~~

::

    # smbmc_example.py
    from smbmc import Client

    # initialise client with connection details
    c = Client(IPMI_SERVER, IPMI_USER, IPMI_PASS)

    # Supermicro BMCs ship with self-signed certificates; to accept them
    # (trusted networks only):
    c = Client(IPMI_SERVER, IPMI_USER, IPMI_PASS, verify=False)

    # retrieve session token (usually lasts 30 minutes)
    # optional: the library maintains session tokens internally.
    c.login()

``Client`` also accepts ``session_timeout`` (minutes, default 30) and
``request_timeout`` (seconds, default 30). Requests release the GIL, so a client
can be shared between threads.

Sensor Metrics
~~~~~~~~~~~~~~

::

    # obtain sensor metrics
    sensors = c.get_sensor_metrics()

    for sensor in sensors:
        print(sensor.__dict__)

    # output (some removed for brevity)
    {'id': 0, 'name': 'System Temp', 'type': <SensorTypeEnum.TEMPERATURE: 1>, 'unit': <SensorUnitEnum.DEGREES_CELSIUS: 1>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 25.0, 'lnr': -9.0, 'lc': -7.0, 'lnc': -5.0, 'unc': 80.0, 'uc': 85.0, 'unr': 90.0}
    {'id': 1, 'name': '12VCC', 'type': <SensorTypeEnum.VOLTAGE: 2>, 'unit': <SensorUnitEnum.VOLTS: 4>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 12.192, 'lnr': 10.144, 'lc': 10.272, 'lnc': 10.784, 'unc': 12.96, 'uc': 13.28, 'unr': 13.408}
    {'id': 9, 'name': 'FAN1', 'type': <SensorTypeEnum.FAN: 4>, 'unit': <SensorUnitEnum.RPM: 18>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': None, 'reading': 3400.0, 'lnr': 400.0, 'lc': 600.0, 'lnc': 800.0, 'unc': 25300.0, 'uc': 25400.0, 'unr': 25500.0}
    {'id': 10, 'name': 'FAN2', 'type': <SensorTypeEnum.FAN: 4>, 'unit': <SensorUnitEnum.RPM: 18>, 'state': <SensorStateEnum.NOT_PRESENT: 2>, 'flags': None, 'reading': 0.0, 'lnr': 400.0, 'lc': 600.0, 'lnc': 800.0, 'unc': 25300.0, 'uc': 25400.0, 'unr': 25500.0}
    {'id': 27, 'name': 'PS2 Status', 'type': <SensorTypeEnum.POWER_SUPPLY: 8>, 'unit': <SensorUnitEnum.UNSPECIFIED: 0>, 'state': <SensorStateEnum.PRESENT: 1>, 'flags': <PowerSupplyFlag.PRESENCE_DETECTED: 1>, 'reading': 0.0, 'lnr': 0.0, 'lc': 0.0, 'lnc': 0.0, 'unc': 0.0, 'uc': 0.0, 'unr': 0.0}

PMBus Metrics
~~~~~~~~~~~~~

::

    # obtain pmbus metrics
    power_supplies = c.get_pmbus_metrics()

    for psu in power_supplies:
        print(psu.__dict__)

    # output
    {'id': 0, 'name': '', 'status': 'ff', 'type': '0', 'input_voltage': 0, 'input_current': 0.0, 'input_power': 0, 'output_voltage': 0.0, 'output_current': 0.0, 'output_power': 0, 'temp_1': 0, 'temp_2': 0, 'fan_1': 0, 'fan_2': 0}
    {'id': 1, 'name': 'FAKE_PSU_SERIAL', 'status': '1', 'type': '1', 'input_voltage': 239, 'input_current': 0.359, 'input_power': 84, 'output_voltage': 12.1, 'output_current': 5.75, 'output_power': 69, 'temp_1': 40, 'temp_2': 55, 'fan_1': 2894, 'fan_2': 3847}


All Metrics
~~~~~~~~~~~

::

    # obtain all metrics
    metrics = c.get_metrics()

    print(metrics)

    # output
    {'pmbus': [], 'sensor': []}

Errors
~~~~~~

All errors derive from ``smbmc.SmbmcError`` (itself an ``Exception``):
``AuthenticationError``, ``HTTPError``, ``TLSError``, ``ResponseError``,
``InvalidURLError`` (also a ``ValueError``) and ``UnsupportedError`` (also a
``NotImplementedError``). ``get_metrics`` raises ``ValueError`` for duplicate or
unknown metric names.

Parsing Only
~~~~~~~~~~~~

Responses fetched by other means can be parsed with ``smbmc.parse_sensors(xml)``
and ``smbmc.parse_power_supplies(xml)``.

Rust Usage
----------

::

    use smbmc::{Client, MetricKind};

    let client = Client::builder("https://192.168.1.1", "ADMIN", "secret")
        .accept_invalid_certs(true) // self-signed BMC certificate
        .build()?;

    let metrics = client.metrics(&MetricKind::ALL).await?;
    for sensor in metrics.sensor.unwrap_or_default() {
        println!("{}: {} {:?}", sensor.name, sensor.reading, sensor.unit);
    }

See the `crate documentation <https://docs.rs/smbmc>`_ and
``crates/smbmc/examples/`` for more, including the blocking client.

Changes in 0.4.0
----------------

- Rewritten in Rust; ``requests`` and ``defusedxml`` are no longer dependencies.
- Readings are rounded to three decimal places instead of truncated, which
  previously could drop 0.001 (e.g. 4.054 instead of 4.055).
- Discrete sensors that are not present, or are of a type other than power
  supply, no longer abort ``get_sensor_metrics``; they are returned with
  ``flags`` set to ``None``.
- Unknown sensor types/units become ``UNKNOWN_<code>`` enum members instead of
  raising ``ValueError``.
- A session dropped by the BMC (e.g. after a reboot) is renewed automatically,
  once, instead of producing a parse error.
- New exception classes (see `Errors`_); login requires a non-empty session ID;
  non-2xx HTTP responses are errors.
- ``Client`` validates the server address on construction and gains
  ``request_timeout`` and ``verify``; it no longer exposes ``password``,
  ``_session`` or ``initial_call``.
- New ``parse_sensors`` and ``parse_power_supplies`` functions.

Development
-----------

Requires `uv <https://docs.astral.sh/uv/>`_, a Rust toolchain and
`just <https://just.systems>`_.

::

    just test          # Python tests (rebuilds the extension when Rust changes)
    just cargo-test    # Rust tests
    just check         # ruff
    just cargo-check   # rustfmt + clippy

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
