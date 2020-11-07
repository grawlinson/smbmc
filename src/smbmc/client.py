"""Provides the Client class."""
from datetime import datetime
from datetime import timedelta

from requests import Session

from .ipmi_pmbus import process_pmbus_response
from .ipmi_sensor import process_sensor_response
from .util import contains_duplicates
from .util import contains_valid_items
from .util import extract_xml_attr

KNOWN_SENSORS = ["pmbus", "sensor"]


class Client:
    """Client used to access Supermicro BMCs."""

    def __init__(self, server, username, password, session_timeout=30):
        """Initialises an instance of smbmc.Client.

        Args:
            server: Address of server in form: 'http://192.168.1.1'.
            username: Username.
            password: Password.
            session_timeout: Session timeout of the BMC (in minutes).
        """
        self.server = server
        self.username = username
        self.password = password
        self._session = Session()
        self.initial_call = datetime(1970, 1, 1)
        self.session_timeout = session_timeout
        self.sid_expiry = timedelta(minutes=self.session_timeout)

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
            self.initial_call = datetime.now()
        else:
            raise Exception("Authentication Error")

    def _query(self, data, path="/cgi/ipmi.cgi"):
        """Query Supermicro BMC.

        Performs session login & token refresh.

        Args:
            path: Path to query. Defaults to '/cgi/ipmi.cgi'.
            data: Requested data.

        Returns:
            request.Response: Response object.
        """
        self._refresh_token()

        return self._session.post(
            f"{self.server}{path}",
            data=data,
        )

    def _refresh_token(self):
        """Refresh SID token if timeout likely."""
        if datetime.now() > (self.initial_call + self.sid_expiry):
            self.login()

    def get_pmbus_metrics(self):
        """Acquire metrics for all power supplies.

        Returns:
            str: XML response.
        """
        r = self._query(
            data={
                "Get_PSInfoReadings.XML": "(0,0)",
            }
        )

        psu_list = extract_xml_attr(r.text, ".//PSItem")
        power_supplies = process_pmbus_response(psu_list)

        return power_supplies

    def get_sensor_metrics(self):
        """Acquire metrics for all sensors.

        Returns:
            str: XML response.
        """
        r = self._query(
            data={
                "SENSOR_INFO.XML": "(1,ff)",
            }
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
