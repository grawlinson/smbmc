"""Integration tests for smbmc.Client, replaying recorded BMC sessions."""

import pytest

from smbmc import AuthenticationError, Client, PowerSupply, Sensor

SMBMC_USER = "ipmi_user"
SMBMC_PASS = "ipmi_pass"


def test_login(replay):
    """Test smbmc.Client.login()."""
    client = Client(replay("login"), SMBMC_USER, SMBMC_PASS)
    client.login()


def test_get_sensor_metrics(replay):
    """Test smbmc.Client.get_sensor_metrics()."""
    client = Client(replay("get_sensor_metrics"), SMBMC_USER, SMBMC_PASS)
    client.login()
    r = client.get_sensor_metrics()

    assert len(r) == 28
    for sensor in r:
        assert isinstance(sensor, Sensor)


def test_get_pmbus_metrics(replay):
    """Test smbmc.Client.get_pmbus_metrics()."""
    client = Client(replay("get_pmbus_metrics"), SMBMC_USER, SMBMC_PASS)
    client.login()
    r = client.get_pmbus_metrics()

    assert len(r) == 4
    for psu in r:
        assert isinstance(psu, PowerSupply)


def test_get_metrics(replay):
    """Test smbmc.Client.get_metrics() logs in automatically, once."""
    client = Client(replay("get_metrics"), SMBMC_USER, SMBMC_PASS)
    r = client.get_metrics()

    assert len(r["pmbus"]) == 4
    assert len(r["sensor"]) == 28


def test_bad_auth(replay):
    """Test invalid authentication."""
    client = Client(replay("bad_auth"), "nonexistent_user", "nonexistent_password")

    with pytest.raises(AuthenticationError, match="authentication failed"):
        client.login()
