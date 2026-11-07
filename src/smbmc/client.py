"""Provides the Client class."""

from __future__ import annotations

from . import _smbmc
from .models import PowerSupply, Sensor
from .util import contains_duplicates, contains_valid_items

KNOWN_SENSORS = ["pmbus", "sensor"]

DEFAULT_SESSION_TIMEOUT = 30
"""Default BMC session timeout, in minutes."""

DEFAULT_REQUEST_TIMEOUT = 30.0
"""Default timeout for each HTTP request, in seconds."""


class Client:
    """Client used to access Supermicro BMCs.

    HTTP requests are made by the native (Rust) extension and release the GIL,
    so a client can be shared between threads.
    """

    def __init__(
        self,
        server: str,
        username: str,
        password: str,
        session_timeout: int | None = None,
        *,
        request_timeout: float | None = None,
        verify: bool = True,
    ) -> None:
        """Initialises an instance of smbmc.Client.

        Args:
            server: Address of server in form: 'http://192.168.1.1'.
            username: Username.
            password: Password.
            session_timeout: Session timeout of the BMC (in minutes).
                default: 30 minutes.
            request_timeout: Timeout for each HTTP request (in seconds).
                default: 30 seconds.
            verify: Verify the BMC's TLS certificate. Supermicro BMCs ship with
                self-signed certificates; set to False only on trusted networks.

        Raises:
            InvalidURLError: The server address is not an http/https URL.
        """
        self.server = server
        self.username = username
        self.session_timeout = (
            DEFAULT_SESSION_TIMEOUT if session_timeout is None else session_timeout
        )
        self.request_timeout = (
            DEFAULT_REQUEST_TIMEOUT if request_timeout is None else request_timeout
        )
        self.verify = verify
        self._client = _smbmc.Client(
            server,
            username,
            password,
            float(self.session_timeout) * 60,
            float(self.request_timeout),
            verify,
        )

    def __repr__(self) -> str:
        """Debug representation, without the password."""
        return f"Client(server={self.server!r}, username={self.username!r})"

    def login(self) -> None:
        """Login to Supermicro web interface.

        Fetches a session ID (SID) cookie, which allows access to the rest
        of the web interface. Calling this is optional: the client logs in
        on first use and again whenever the session timeout has elapsed.

        Raises:
            AuthenticationError: The BMC did not issue a session ID.
            HTTPError: The request failed.
        """
        self._client.login()

    def get_pmbus_metrics(self) -> list[PowerSupply]:
        """Acquire metrics for all power supplies.

        Returns:
            list[PowerSupply]: All power supplies available on the PMBus
            interface.
        """
        return [PowerSupply._from_raw(raw) for raw in self._client.pmbus_metrics()]

    def get_sensor_metrics(self) -> list[Sensor]:
        """Acquire metrics for all sensors.

        Returns:
            list[Sensor]: A list of all sensors available to the BMC.
        """
        return [Sensor._from_raw(raw) for raw in self._client.sensor_metrics()]

    def get_metrics(self, metrics: list[str] | None = None) -> dict[str, list]:
        """Fetch all metrics available.

        Args:
            metrics: List of metric(s) to query. Defaults to ["pmbus", "sensor"]

        Raises:
            ValueError: Argument contains duplicate or invalid metrics.

        Returns:
            dict: A dict containing all metrics.
        """
        if metrics is None:
            metrics = ["pmbus", "sensor"]

        if contains_duplicates(metrics):
            raise ValueError("metrics array contains duplicates")

        if not contains_valid_items(KNOWN_SENSORS, metrics):
            raise ValueError("metrics array contains invalid metrics")

        fetch = {
            "pmbus": self.get_pmbus_metrics,
            "sensor": self.get_sensor_metrics,
        }
        return {metric: fetch[metric]() for metric in metrics}
