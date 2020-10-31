"""Unit tests for utility functions."""
import pytest

from smbmc.util import contains_duplicates
from smbmc.util import contains_valid_items
from smbmc.util import extract_xml_attr
from smbmc.util import hex_signed_int
from smbmc.util import signed_int
from smbmc.util import ten_bit_str


@pytest.mark.parametrize(
    "items,expected_result",
    [
        (
            ["first", "second", "third"],
            False,
        ),
        (
            ["duplicate", "duplicate", "duplicate"],
            True,
        ),
    ],
)
def test_contains_duplicates(items, expected_result):
    """Check if duplicates are correctly detected.

    Args:
        items: Array of items to check.
        expected_result: Boolean value representing duplicate status.
    """
    assert contains_duplicates(items) is expected_result


@pytest.mark.parametrize(
    "allowed_items,items,expected_result",
    [
        (
            ["bang-a-rang!", "rufiooooo!"],
            ["definitely not rufio!"],
            False,
        ),
        (
            ["1", "2", "3"],
            ["2", "3"],
            True,
        ),
    ],
)
def test_contains_valid_items(allowed_items, items, expected_result):
    """Check if valid/invalid items are correctly detected.

    Args:
        allowed_items: Array of items allowed to be present.
        items: Array of items to check.
        expected_result: Boolean value representing validity of items array.
    """
    assert contains_valid_items(allowed_items, items) is expected_result


@pytest.mark.parametrize(
    "unsigned_value,signed_bit,signed_value",
    [
        (129, 8, -127),
        (874, 10, -150),
        (200, 0, 200),
        (7, 4, 7),
    ],
)
def test_signed_int(unsigned_value, signed_bit, signed_value):
    """Check conversion of unsigned -> signed integers.

    Args:
        unsigned_value: Unsigned integer.
        signed_bit: Bit representing integer length, e.g. 8 = 8-bit integer.
        signed_value: Signed integer representing expected result.
    """
    assert signed_int(unsigned_value, signed_bit) == signed_value


@pytest.mark.parametrize(
    "unsigned_string,signed_bit,hex_string",
    [
        ("10", 0, "0x10"),
        ("120", 8, "0x20"),
    ],
)
def test_hex_signed_int(unsigned_string, signed_bit, hex_string):
    """Check conversion of unsigned strings to signed integers stored as hex strings.

    Args:
        unsigned_string: Unsigned integer stored as a string.
        signed_bit: Bit representing integer length.
        hex_string: Signed integer stored as a hex string.
    """
    assert hex_signed_int(unsigned_string, signed_bit) == hex_string


@pytest.mark.parametrize(
    "two_byte_string,ten_bit_int",
    [
        ("FF", 768),
        ("AB", 512),
        ("200", 2),
        ("0x200", 2),
    ],
)
def test_ten_bit_str(two_byte_string, ten_bit_int):
    """Check conversion of 16-bit hexadecimal strings to 10-bit integers.

    Args:
        two_byte_string: Hexadecimal string representing an integer value to convert.
        ten_bit_int: Integer representing 10-bit result.

    """
    assert ten_bit_str(two_byte_string) == ten_bit_int


@pytest.mark.parametrize(
    "xml_file,selector,expected_length",
    [
        (
            "ipmi_response_sensors",
            ".//SENSOR",
            28,
        ),
        (
            "ipmi_response_sensors",
            ".//NOT_A_SENSOR",
            0,
        ),
        (
            "ipmi_response_sel",
            ".//SEL",
            6,
        ),
        (
            "ipmi_response_pmbus",
            ".//PSItem",
            4,
        ),
        (
            "ipmi_response_pmbus",
            ".//PSInfo",
            1,
        ),
    ],
)
def test_extract_xml(xml_file, selector, expected_length):
    """Ensure that specific selectors are extracted.

    Args:
        xml_file: XML file containing an IPMI response.
        selector: XML Selector used to match specific sub-element(s).
        expected_length: Quantity of expected sub-element(s).
    """
    xml_string = open(f"tests/unit/{xml_file}.xml").read()
    extracted_list = extract_xml_attr(xml_string, selector)

    assert extracted_list is not None
    assert len(extracted_list) == expected_length
