//! Error type.

/// Convenience alias for `Result<T, smbmc::Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors produced by this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The HTTP request failed or the BMC returned an error status.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The BMC did not issue a session ID; the credentials are probably wrong.
    #[error("authentication failed: the BMC did not issue a session ID")]
    AuthenticationFailed,

    /// The TLS configuration could not be built (e.g. the platform trust
    /// store could not be loaded).
    #[error("TLS configuration failed: {0}")]
    Tls(String),

    /// The server address could not be parsed as an `http`/`https` URL.
    #[error("invalid server address {0:?}: expected e.g. \"https://192.168.1.1\"")]
    InvalidUrl(String),

    /// The BMC response was not well-formed XML (or contained a DTD).
    #[error("malformed XML response: {0}")]
    Xml(#[from] roxmltree::Error),

    /// The response was XML but not an `<IPMI>` document, e.g. the BMC's
    /// login page after the session was dropped. Holds the root element name.
    #[error("unexpected response from the BMC: expected <IPMI>, got <{0}>")]
    UnexpectedResponse(String),

    /// An element in the BMC response lacked a required attribute.
    #[error("<{element}> is missing the `{attribute}` attribute")]
    MissingAttribute {
        /// Tag name of the element.
        element: String,
        /// Name of the missing attribute.
        attribute: &'static str,
    },

    /// A field expected to be hexadecimal could not be parsed.
    #[error("invalid hexadecimal value {value:?}")]
    InvalidHex {
        /// The offending value.
        value: String,
    },

    /// The sensor uses an IPMI linearisation formula that is not implemented.
    #[error("unsupported linearisation formula {0:#04x}")]
    UnsupportedLinearisation(u8),
}
