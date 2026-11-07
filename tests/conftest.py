"""Test configuration.

Integration tests replay the betamax cassettes recorded against a real BMC
using a local HTTP server (pytest-httpserver), since HTTP is now performed
by the native extension rather than ``requests``.
"""

import json
from pathlib import Path

import pytest
from werkzeug import Response

# Fixtures are shared with (and shipped in) the Rust crate.
FIXTURES = Path(__file__).parents[1] / "crates" / "smbmc" / "tests" / "fixtures"
CASSETTES = FIXTURES / "cassettes"
SMBMC_USER = "ipmi_user"
SMBMC_PASS = "ipmi_pass"


@pytest.fixture
def replay(httpserver):
    """Return a function that loads a cassette into the HTTP server.

    Each recorded request must arrive in order, with the same path, method,
    form body and ``Cookie`` header. The returned base URL is for the client.
    """

    def load(name: str) -> str:
        cassette = json.loads((CASSETTES / f"Client_{name}.json").read_text())
        for interaction in cassette["http_interactions"]:
            request = interaction["request"]
            response = interaction["response"]
            body = (
                request["body"]["string"]
                .replace("<USER>", SMBMC_USER)
                .replace("<PASS>", SMBMC_PASS)
            )
            headers = {}
            if "Cookie" in request["headers"]:
                headers["Cookie"] = request["headers"]["Cookie"][0]
            httpserver.expect_ordered_request(
                request["uri"].removeprefix("<SERVER>"),
                method=request["method"],
                data=body,
                headers=headers,
            ).respond_with_response(
                Response(
                    response["body"]["string"],
                    status=response["status"]["code"],
                    headers=[
                        (name, value)
                        for name, values in response["headers"].items()
                        for value in values
                        if name.lower() not in ("content-length", "transfer-encoding")
                    ],
                )
            )
        return httpserver.url_for("").rstrip("/")

    yield load
    httpserver.check_assertions()
    assert not httpserver.ordered_handlers, "not every recorded request was made"
