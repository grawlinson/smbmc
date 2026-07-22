"""Provides the Client class."""

from datetime import datetime, timedelta, timezone

import requests

from .ipmi_pmbus import process_pmbus_response
from .ipmi_sensor import process_sensor_response
from .models import PowerSupply, Sensor
from .util import contains_duplicates, contains_valid_items, extract_xml_attr

KNOWN_SENSORS = ["pmbus", "sensor"]


class Client:
    """Client used to access Supermicro BMCs."""

    def __init__(
        self,
        server: str,
        username: str,
        password: str,
        session_timeout: int | None = None,
    ) -> None:
        """Initialises an instance of smbmc.Client.

        Args:
            server: Address of server in form: 'http://192.168.1.1'.
            username: Username.
            password: Password.
            session_timeout: Session timeout of the BMC (in minutes).
                default: 30 minutes.
        """
        self.server = server
        self.username = username
        self.password = password
        self._session = requests.Session()
        self.initial_call = datetime(1970, 1, 1, tzinfo=timezone.utc)
        if session_timeout is None:
            self.session_timeout = 30
        else:
            self.session_timeout = session_timeout
        self.sid_expiry = timedelta(minutes=self.session_timeout)

    def login(self) -> None:
        """Login to Supermicro web interface.

        Fetches a session ID (SID) cookie, which allows access to the rest
        of the web interface. SID length is approximately 30 minutes,
        according to the default timeout configuration.

        Raises:
            Exception: Authentication Error.
        """
        self._session.post(
            f"{self.server}/cgi/login.cgi",
            data={
                "name": self.username,
                "pwd": self.password,
            },
        )

        if "SID" in self._session.cookies.get_dict():
            self.initial_call = datetime.now(tz=timezone.utc)
        else:
            raise Exception("Authentication Error")

    def _query(self, data: dict, path: str | None = None) -> requests.Response:
        """Query Supermicro BMC.

        Performs session login & token refresh.

        Args:
            path: Path to query. Defaults to '/cgi/ipmi.cgi'.
            data: Requested data.

        Returns:
            requests.Response: Response object.
        """
        if path is None:
            path = "/cgi/ipmi.cgi"

        self._refresh_token()

        return self._session.post(
            f"{self.server}{path}",
            data=data,
        )

    def _refresh_token(self) -> None:
        """Refresh SID token if timeout likely."""
        if datetime.now(tz=timezone.utc) > (self.initial_call + self.sid_expiry):
            self.login()

    def get_pmbus_metrics(self) -> list[PowerSupply]:
        """Acquire metrics for all power supplies.

        Returns:
            list[PowerSupply]: All power supplies available on the PMBus
            interface.
        """
        r = self._query(
            data={
                "Get_PSInfoReadings.XML": "(0,0)",
            }
        )

        psu_list = extract_xml_attr(r.text, ".//PSItem")
        power_supplies = process_pmbus_response(psu_list)

        return power_supplies

    def get_sensor_metrics(self) -> list[Sensor]:
        """Acquire metrics for all sensors.

        Returns:
            list[Sensor]: A list of all sensors available to the BMC.
        """
        r = self._query(
            data={
                "SENSOR_INFO.XML": "(1,ff)",
            }
        )

        sensor_list = extract_xml_attr(r.text, ".//SENSOR")
        sensors = process_sensor_response(sensor_list)

        return sensors

    def get_metrics(self, metrics: dict | None = None) -> dict:
        """Fetch all metrics available.

        Args:
            metrics: List of metric(s) to query. Defaults to ["pmbus", "sensor"]

        Raises:
            Exception: Argument contains duplicate metrics.
            Exception: Argument contains invalid metrics.

        Returns:
            dict: A dict containing all metrics.
        """
        if metrics is None:
            metrics = ["pmbus", "sensor"]

        if contains_duplicates(metrics):
            raise Exception("metrics array contains duplicates")

        if not contains_valid_items(KNOWN_SENSORS, metrics):
            raise Exception("metrics array contains invalid metrics")

        # self.login()
        result = {}

        for metric in metrics:
            values = None
            if metric == "pmbus":
                values = self.get_pmbus_metrics()
            elif metric == "sensor":  # pragma: no cover
                values = self.get_sensor_metrics()

            result.update({metric: values})

        return result
