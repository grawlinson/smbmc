"""Test Configuration."""
import os

import betamax

SMBMC_SERVER = os.environ.get("SMBMC_SERVER", "http://192.168.1.1")
SMBMC_USER = os.environ.get("SMBMC_USER", "ipmi_user")
SMBMC_PASS = os.environ.get("SMBMC_PASS", "ipmi_pass")

with betamax.Betamax.configure() as config:
    config.cassette_library_dir = "tests/integration/cassettes"
    config.default_cassette_options["match_requests_on"].extend(["body", "path"])
    config.define_cassette_placeholder("<SERVER>", SMBMC_SERVER)
    config.define_cassette_placeholder("<USER>", SMBMC_USER)
    config.define_cassette_placeholder("<PASS>", SMBMC_PASS)
