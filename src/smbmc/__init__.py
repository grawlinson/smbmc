"""The smbmc package."""
try:
    from importlib.metadata import version, PackageNotFoundError
except ImportError:  # pragma: no cover
    from importlib_metadata import version, PackageNotFoundError

try:
    __version__ = version(__name__)
except PackageNotFoundError:  # pragma: no cover
    __version__ = "unknown"

from .models import (
    PowerSupply,
    PowerSupplyFlag,
    Sensor,
    SensorStateEnum,
    SensorTypeEnum,
    SensorUnitEnum,
)
from .client import Client
