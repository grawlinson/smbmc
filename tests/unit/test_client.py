"""Unit tests for smbmc.Client class."""
import pytest

from smbmc import Client

# TODO stub out request call
client = Client("", "", "")


def test_duplicate_metrics():
    """Check for duplicate metrics."""
    with pytest.raises(Exception, match="duplicates"):
        assert client.get_metrics(["dupe", "dupe"])


def test_invalid_metrics():
    """Check for invalid metrics."""
    with pytest.raises(Exception, match="invalid metric"):
        assert client.get_metrics([None, 1, "magic_school_bus"])
