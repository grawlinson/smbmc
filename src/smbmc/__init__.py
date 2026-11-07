"""The smbmc package.

An unofficial interface for obtaining metrics from Supermicro BMCs, backed by
the ``smbmc`` Rust crate.
"""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version(__name__)
except PackageNotFoundError:  # pragma: no cover
    __version__ = "unknown"

from . import _smbmc
from .client import Client as Client
from .exceptions import (
    AuthenticationError as AuthenticationError,
)
from .exceptions import (
    HTTPError as HTTPError,
)
from .exceptions import (
    InvalidURLError as InvalidURLError,
)
from .exceptions import (
    ResponseError as ResponseError,
)
from .exceptions import (
    SmbmcError as SmbmcError,
)
from .exceptions import (
    TLSError as TLSError,
)
from .exceptions import (
    UnsupportedError as UnsupportedError,
)
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


def parse_sensors(xml: str) -> list[Sensor]:
    """Parse a ``SENSOR_INFO.XML`` response obtained by other means.

    Args:
        xml: The response body.

    Returns:
        list[Sensor]: The sensors in the response.

    Raises:
        ResponseError: The response could not be parsed.
    """
    return [Sensor._from_raw(raw) for raw in _smbmc.parse_sensors(xml)]


def parse_power_supplies(xml: str) -> list[PowerSupply]:
    """Parse a ``Get_PSInfoReadings.XML`` response obtained by other means.

    Args:
        xml: The response body.

    Returns:
        list[PowerSupply]: The power supplies in the response.

    Raises:
        ResponseError: The response could not be parsed.
    """
    return [PowerSupply._from_raw(raw) for raw in _smbmc.parse_power_supplies(xml)]
