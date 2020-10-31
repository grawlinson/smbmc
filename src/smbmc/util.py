"""Provides utility functions."""
# from __future__ import annotations  # only works with python 3.7+
# list[str]


def contains_duplicates(item_list: list) -> bool:
    """Check if given list contains any duplicates.

    Args:
        item_list: List of items to check for duplicates.

    Returns:
        True if list contains duplicates.
    """
    element_set = set()
    for item in item_list:
        if item in element_set:
            return True
        else:
            element_set.add(item)
    return False


def contains_valid_items(known_items: list, item_list: list) -> bool:
    """Check if given list contains known items.

    Args:
        known_items: List of known strings to check against.
        item_list: List of strings to check for unknown items.

    Returns:
        True if list contains valid items.
    """
    for item in item_list:
        if item not in known_items:
            return False
    return True


def signed_int(value: int, signed_bit: int) -> int:
    """Convert from unsigned to a signed integer.

    Args:
        value: Unsigned integer.
        signed_bit: Location of the signed bit.

    Returns:
        int: Signed integer.
    """
    if signed_bit > 0:
        if (value % (0x01 << signed_bit) / (0x01 << (signed_bit - 1))) < 1:
            return value % (0x01 << signed_bit - 1)
        else:
            temporary_value = (value % (0x01 << signed_bit - 1)) ^ (
                (0x01 << signed_bit - 1) - 1
            )
            return -1 - temporary_value

    return value


def hex_signed_int(value: str, signed_bit=8) -> str:
    """Convert from unsigned to signed integer as a hexadecimal string.

    Args:
        value: Unsigned integer.
        signed_bit: Location of the signed bit. Default: 8.

    Returns:
        str: Hexadecimal representation of a signed integer.
    """
    return hex(signed_int(int(value, 16), signed_bit))


def ten_bit_str(value: str) -> int:
    """Convert two bytes to a 10-bit int.

    TODO: check this, I'm tired.

    Args:
        value: String consisting of two bytes.

    Returns:
        int: 10-bit int.
    """
    return ((int(value, 16) & 0xC0) << 2) + (int(value, 16) >> 8)


def extract_xml_attr(xml: str, match: str) -> list:
    """Extract all incidences of a given XML element.

    Args:
        xml: String representation of an XML document.
        match: Subelements to match via tag name or path.

    Returns:
        list: List of subelements matching query.
    """
    from defusedxml import ElementTree

    tree = ElementTree.fromstring(xml.strip())
    elements = tree.findall(match)

    result = []
    for element in elements:
        result.append(element.attrib)

    return result
