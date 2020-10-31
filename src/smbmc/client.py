"""Provides the Client class."""
from time import time as now

from requests import Session

from .ipmi_pmbus import process_pmbus_response
from .ipmi_sensor import process_sensor_response
from .util import contains_duplicates
from .util import contains_valid_items
from .util import extract_xml_attr

KNOWN_SENSORS = ["pmbus", "sensor"]


class Client:
    """Client used to access Supermicro BMCs."""

    def __init__(self, server, username, password):
        """Initialises an instance of smbmc.Client.

        Args:
            server: Address of server in form: 'http://192.168.1.1'.
            username: Username.
            password: Password.
        """
        self.server = server
        self.username = username
        self.password = password
        self._session = Session()
        self.last_call = None

    def login(self):
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

        if "SID" in self._session.cookies.get_dict().keys():
            self.last_call = now()
        else:
            raise Exception("Authentication Error")

    def get_pmbus_metrics(self):
        """Acquire metrics for all power supplies.

        Returns:
            str: XML response.
        """
        self.last_call = now()
        r = self._session.post(
            f"{self.server}/cgi/ipmi.cgi",
            data={
                "Get_PSInfoReadings.XML": "(0,0)",
            },
        )

        psu_list = extract_xml_attr(r.text, ".//PSItem")
        power_supplies = process_pmbus_response(psu_list)

        return power_supplies

    def get_sensor_metrics(self):
        """Acquire metrics for all sensors.

        Returns:
            str: XML response.
        """
        self.last_call = now()

        r = self._session.post(
            f"{self.server}/cgi/ipmi.cgi",
            data={
                "SENSOR_INFO.XML": "(1,ff)",
            },
        )

        sensor_list = extract_xml_attr(r.text, ".//SENSOR")
        sensors = process_sensor_response(sensor_list)

        return sensors

    def get_metrics(self, metrics=["pmbus", "sensor"]):  # noqa: C901
        """Fetch metrics with minimum network calls.

        Args:
            metrics: List of metric(s) to query.

        Raises:
            Exception: Argument contains duplicate metrics.
            Exception: Argument contains invalid metrics.

        Returns:
            dict: A dict containing all metrics.
        """
        if contains_duplicates(metrics):
            raise Exception("metrics array contains duplicates")

        if not contains_valid_items(KNOWN_SENSORS, metrics):
            raise Exception("metrics array contains invalid metrics")

        self.login()
        result = {}

        for metric in metrics:
            values = None
            if metric == "pmbus":
                values = self.get_pmbus_metrics()
            elif metric == "sensor":  # pragma: no cover
                values = self.get_sensor_metrics()

            result.update({metric: values})

        return result
