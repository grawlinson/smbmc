"""Exceptions raised by smbmc.

Every exception derives from :class:`SmbmcError`, which itself derives from
:class:`Exception`, so existing ``except Exception`` handlers keep working.
"""


class SmbmcError(Exception):
    """Base class for all smbmc errors."""


class AuthenticationError(SmbmcError):
    """The BMC did not issue a session ID; the credentials are probably wrong."""


class HTTPError(SmbmcError):
    """The HTTP request failed, timed out, or returned an error status."""


class TLSError(SmbmcError):
    """The TLS configuration could not be built."""


class InvalidURLError(SmbmcError, ValueError):
    """The server address is not an ``http``/``https`` URL."""


class ResponseError(SmbmcError):
    """The BMC response could not be parsed."""


class UnsupportedError(SmbmcError, NotImplementedError):
    """The BMC uses a feature (e.g. a linearisation formula) not yet supported."""
