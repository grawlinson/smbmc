"""The smbmc package."""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version(__name__)
except PackageNotFoundError:  # pragma: no cover
    __version__ = "unknown"

from .client import Client as Client
from .models import (
    PowerSupply as PowerSupply,
)
from .models import (
    PowerSupplyFlag as PowerSupplyFlag,
)
from .models import (
    Sensor as Sensor,
)
from .models import (
    SensorStateEnum as SensorStateEnum,
)
from .models import (
    SensorTypeEnum as SensorTypeEnum,
)
from .models import (
    SensorUnitEnum as SensorUnitEnum,
)
