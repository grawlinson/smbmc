"""Type stubs for the native extension module (internal; use ``smbmc``)."""

from typing import Any

class Client:
    def __init__(
        self,
        server: str,
        username: str,
        password: str,
        session_timeout: float,
        request_timeout: float,
        verify: bool,
    ) -> None: ...
    def login(self) -> None: ...
    def pmbus_metrics(self) -> list[dict[str, Any]]: ...
    def sensor_metrics(self) -> list[dict[str, Any]]: ...

def parse_sensors(xml: str) -> list[dict[str, Any]]: ...
def parse_power_supplies(xml: str) -> list[dict[str, Any]]: ...
