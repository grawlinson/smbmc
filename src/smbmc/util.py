"""Provides utility functions."""

from collections.abc import Iterable


def contains_duplicates(item_list: Iterable[object]) -> bool:
    """Check if given list contains any duplicates.

    Args:
        item_list: List of items to check for duplicates.

    Returns:
        True if list contains duplicates.
    """
    element_set: set[object] = set()
    for item in item_list:
        if item in element_set:
            return True
        element_set.add(item)
    return False


def contains_valid_items(known_items: list, item_list: Iterable[object]) -> bool:
    """Check if given list contains known items.

    Args:
        known_items: List of known strings to check against.
        item_list: List of strings to check for unknown items.

    Returns:
        True if list contains valid items.
    """
    return all(item in known_items for item in item_list)
