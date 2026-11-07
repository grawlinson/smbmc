"""Unit tests for utility functions."""

import pytest

from smbmc.util import contains_duplicates, contains_valid_items


@pytest.mark.parametrize(
    ("items", "expected_result"),
    [
        (["first", "second", "third"], False),
        (["duplicate", "duplicate", "duplicate"], True),
    ],
)
def test_contains_duplicates(items, expected_result):
    """Check if duplicates are correctly detected."""
    assert contains_duplicates(items) is expected_result


@pytest.mark.parametrize(
    ("allowed_items", "items", "expected_result"),
    [
        (["bang-a-rang!", "rufiooooo!"], ["definitely not rufio!"], False),
        (["1", "2", "3"], ["2", "3"], True),
    ],
)
def test_contains_valid_items(allowed_items, items, expected_result):
    """Check if valid/invalid items are correctly detected."""
    assert contains_valid_items(allowed_items, items) is expected_result
