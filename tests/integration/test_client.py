"""Integration tests for smbmc.Client class."""
import os

import betamax
import pytest

from smbmc import Client
from smbmc import PowerSupply
from smbmc import Sensor

SMBMC_SERVER = os.environ.get("SMBMC_SERVER", "http://192.168.1.1")
SMBMC_USER = os.environ.get("SMBMC_USER", "ipmi_user")
SMBMC_PASS = os.environ.get("SMBMC_PASS", "ipmi_pass")


class TestClient:
    """Testing class for smbmc.Client."""

    @pytest.fixture(autouse=True)
    def setup(self):
        """Set-up for testing."""
        self.client = Client(SMBMC_SERVER, SMBMC_USER, SMBMC_PASS)
        self.recorder = betamax.Betamax(self.client._session)

    @staticmethod
    def generate_cassette_name(method_name):
        """Generate cassette name.

        Args:
            method_name: The method name being tested.

        Returns:
            cassette_name: Name used for betamax Cassette.
        """
        return f"Client_{method_name}"

    def test_login(self):
        """Test smbmc.Client.login()."""
        cassette_name = self.generate_cassette_name("login")
        with self.recorder.use_cassette(cassette_name):
            self.client.login()

        assert self.client.last_call is not None
        assert "SID" in self.client._session.cookies.get_dict().keys()
        assert self.client._session.cookies["SID"] is not None

    def test_get_sensor_metrics(self):
        """Test smbmc.Client.get_sensor_metrics()."""
        cassette_name = self.generate_cassette_name("get_sensor_metrics")
        with self.recorder.use_cassette(cassette_name):
            self.client.login()
            r = self.client.get_sensor_metrics()

        assert r is not None
        assert len(r) == 28
        for sensor in r:
            assert isinstance(sensor, Sensor)

    def test_get_pmbus_metrics(self):
        """Test smbmc.Client.get_pmbus_metrics()."""
        cassette_name = self.generate_cassette_name("get_pmbus_metrics")
        with self.recorder.use_cassette(cassette_name):
            self.client.login()
            r = self.client.get_pmbus_metrics()

        assert r is not None
        assert len(r) == 4
        for psu in r:
            assert isinstance(psu, PowerSupply)

    def test_get_metrics(self):
        """Test smbmc.Client.get_metrics()."""
        cassette_name = self.generate_cassette_name("get_metrics")
        with self.recorder.use_cassette(cassette_name):
            r = self.client.get_metrics()

        assert r is not None
        assert "pmbus" in r
        assert r["pmbus"] is not None
        assert len(r["pmbus"]) == 4
        assert "sensor" in r
        assert r["sensor"] is not None
        assert len(r["sensor"]) == 28

    def test_bad_auth(self):
        """Test invalid authentication."""
        self.client = Client(SMBMC_SERVER, "nonexistent_user", "nonexistent_password")
        self.recorder = betamax.Betamax(self.client._session)

        cassette_name = self.generate_cassette_name("bad_auth")
        with self.recorder.use_cassette(cassette_name):
            with pytest.raises(Exception, match="Authentication Error"):
                assert self.client.login()
