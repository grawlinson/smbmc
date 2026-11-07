"""Unit tests for PMBus parsing (performed by the native extension)."""

import json
from pathlib import Path

from smbmc import PowerSupply, parse_power_supplies

FIXTURES = Path(__file__).parents[2] / "crates" / "smbmc" / "tests" / "fixtures"


def test_process_power_supplies():
    """Test the processing of an IPMI response containing PSUs."""
    power_supplies = parse_power_supplies(
        (FIXTURES / "ipmi_response_pmbus.xml").read_text()
    )

    assert len(power_supplies) == 4

    for psu in power_supplies:
        assert isinstance(psu, PowerSupply)
        if psu.id == 1:
            assert psu.name == "PSU0SERIAL0NO00"
            assert psu.status == "1"
            assert psu.type == "1"
            assert psu.input_voltage == 239
            assert psu.input_current == 0.359
            assert psu.input_power == 84
            assert psu.output_voltage == 12.1
            assert psu.output_current == 5.75
            assert psu.output_power == 69
            assert psu.temp_1 == 40
            assert psu.temp_2 == 55
            assert psu.fan_1 == 2894
            assert psu.fan_2 == 3847
        else:
            assert psu.name == ""
            assert psu.status == "ff"
            assert psu.type == "0"
            assert psu.input_voltage == 0
            assert psu.input_current == 0.0
            assert psu.output_power == 0
            assert psu.fan_2 == 0


def test_matches_python_0_3_0():
    """Results are identical to the pure-Python 0.3.0 implementation."""
    psus = parse_power_supplies((FIXTURES / "ipmi_response_pmbus.xml").read_text())
    golden = json.loads((FIXTURES / "golden_pmbus_v0.3.0.json").read_text())

    assert [p.__dict__ for p in psus] == golden


def test_repr():
    """repr() shows the attributes."""
    (psu, *_) = parse_power_supplies((FIXTURES / "ipmi_response_pmbus.xml").read_text())
    assert repr(psu).startswith("PowerSupply({'id': 0")
