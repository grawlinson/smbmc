===============
API Reference
===============

Classes
=======

Client
------

.. autoclass:: smbmc.Client
   :members:
   :inherited-members:

Parsing
-------

Responses obtained by other means can be parsed without a ``Client``.

.. autofunction:: smbmc.parse_sensors

.. autofunction:: smbmc.parse_power_supplies

Sensor
------

.. autoclass:: smbmc.models.Sensor

Power Supply
------------

.. autoclass:: smbmc.models.PowerSupply

Enums & Flags
=============

SensorStateEnum
---------------

.. autoclass:: smbmc.models.SensorStateEnum

SensorUnitEnum
--------------

.. autoclass:: smbmc.models.SensorUnitEnum

SensorTypeEnum
--------------

.. autoclass:: smbmc.models.SensorTypeEnum

PowerSupplyFlag
---------------

.. autoclass:: smbmc.models.PowerSupplyFlag

Exceptions
==========

.. automodule:: smbmc.exceptions
   :members:
   :show-inheritance:
