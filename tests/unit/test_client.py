"""Unit tests for smbmc.Client class."""

import pytest

from smbmc import Client, HTTPError, InvalidURLError, SmbmcError

client = Client("http://localhost", "", "")


def test_duplicate_metrics():
    """Check for duplicate metrics."""
    with pytest.raises(ValueError, match="duplicates"):
        assert client.get_metrics(["dupe", "dupe"])


def test_invalid_metrics():
    """Check for invalid metrics."""
    with pytest.raises(ValueError, match="invalid metric"):
        assert client.get_metrics([None, 1, "magic_school_bus"])


@pytest.mark.parametrize("server", ["", "192.168.1.1", "ftp://host"])
def test_invalid_server(server):
    """Server addresses are validated up front."""
    with pytest.raises(InvalidURLError) as exc_info:
        Client(server, "user", "pass")
    assert isinstance(exc_info.value, ValueError)
    assert isinstance(exc_info.value, SmbmcError)


@pytest.mark.parametrize(
    ("kwargs", "message"),
    [
        ({"session_timeout": -1}, "session_timeout"),
        ({"request_timeout": -1.0}, "request_timeout"),
    ],
)
def test_invalid_timeouts(kwargs, message):
    """Negative timeouts are rejected."""
    with pytest.raises(ValueError, match=message):
        Client("http://localhost", "user", "pass", **kwargs)


def test_defaults_and_repr():
    """Defaults match 0.3.x and the password is not exposed."""
    c = Client("https://bmc", "admin", "hunter2", verify=False)
    assert c.session_timeout == 30
    assert c.request_timeout == 30.0
    assert c.verify is False
    assert "hunter2" not in repr(c)
    assert not hasattr(c, "password")


def test_connection_error():
    """Transport failures raise HTTPError."""
    c = Client("http://127.0.0.1:9", "user", "pass", request_timeout=2)
    with pytest.raises(HTTPError):
        c.login()


def test_session_refresh(httpserver):
    """With a zero session timeout, every query logs in again."""
    httpserver.expect_request("/cgi/login.cgi").respond_with_response(
        _login_response("abc")
    )
    httpserver.expect_request(
        "/cgi/ipmi.cgi", headers={"Cookie": "SID=abc"}
    ).respond_with_data("<IPMI/>")
    c = Client(httpserver.url_for("").rstrip("/"), "u", "p", session_timeout=0)
    assert c.get_pmbus_metrics() == []
    assert c.get_sensor_metrics() == []
    logins = [r for r, _ in httpserver.log if r.path == "/cgi/login.cgi"]
    assert len(logins) == 2


def _login_response(sid):
    from werkzeug import Response

    return Response(headers=[("Set-Cookie", f"SID={sid}; path=/")])
