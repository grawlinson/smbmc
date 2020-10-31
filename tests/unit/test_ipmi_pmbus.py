"""Unit tests for IPMI PMBus functions."""
from smbmc.ipmi_pmbus import process_pmbus_response
from smbmc.util import extract_xml_attr


def test_process_power_supplies():
    """Test the processing of an IPMI response containing PSUs."""
    xml_file = "ipmi_response_pmbus"
    selector = ".//PSItem"
    xml_string = open(f"tests/unit/{xml_file}.xml").read()
    extracted_list = extract_xml_attr(xml_string, selector)
    power_supplies = process_pmbus_response(extracted_list)

    assert len(power_supplies) == 4

    for psu in power_supplies:
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
            assert psu.input_power == 0
            assert psu.output_voltage == 0.0
            assert psu.output_current == 0.0
            assert psu.output_power == 0
            assert psu.temp_1 == 0
            assert psu.temp_2 == 0
            assert psu.fan_1 == 0
            assert psu.fan_2 == 0
