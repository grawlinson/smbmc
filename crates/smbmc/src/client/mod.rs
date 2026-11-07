//! HTTP clients for the BMC web interface (port of `client.py`).
//!
//! The async [`Client`](crate::Client) and the blocking
//! [`blocking::Client`](crate::blocking::Client) share the configuration,
//! session handling and request definitions in this module.

pub(crate) mod async_impl;
#[cfg(feature = "blocking")]
pub(crate) mod blocking;
#[cfg(feature = "rustls")]
mod tls;

use std::fmt;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, SET_COOKIE};

use crate::error::{Error, Result};
use crate::models::{PowerSupply, Sensor};

/// Default BMC session lifetime (the Supermicro default is 30 minutes).
pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Default timeout for each HTTP request.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const LOGIN_PATH: &str = "/cgi/login.cgi";
pub(crate) const IPMI_PATH: &str = "/cgi/ipmi.cgi";
pub(crate) const PMBUS_QUERY: (&str, &str) = ("Get_PSInfoReadings.XML", "(0,0)");
pub(crate) const SENSOR_QUERY: (&str, &str) = ("SENSOR_INFO.XML", "(1,ff)");
const SESSION_COOKIE: &str = "SID";

/// A category of metrics that can be requested with `metrics()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum MetricKind {
    /// Power supplies, via PMBus.
    Pmbus,
    /// IPMI sensors.
    Sensor,
}

impl MetricKind {
    /// Every metric kind, in the order they are fetched.
    pub const ALL: [Self; 2] = [Self::Pmbus, Self::Sensor];
}

/// Result of `metrics()`: each field is `Some` if that kind was requested.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Metrics {
    /// Power supplies, if [`MetricKind::Pmbus`] was requested.
    pub pmbus: Option<Vec<PowerSupply>>,
    /// Sensors, if [`MetricKind::Sensor`] was requested.
    pub sensor: Option<Vec<Sensor>>,
}

/// Builder for [`Client`](crate::Client) and
/// [`blocking::Client`](crate::blocking::Client).
///
/// ```no_run
/// # async fn run() -> smbmc::Result<()> {
/// use std::time::Duration;
///
/// let client = smbmc::Client::builder("https://192.168.1.1", "ADMIN", "secret")
///     .session_timeout(Duration::from_secs(10 * 60))
///     .accept_invalid_certs(true) // BMCs commonly use self-signed certificates
///     .build()?;
/// let sensors = client.sensor_metrics().await?;
/// # Ok(())
/// # }
/// ```
#[must_use]
pub struct ClientBuilder {
    pub(crate) server: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) session_timeout: Duration,
    pub(crate) request_timeout: Duration,
    pub(crate) accept_invalid_certs: bool,
}

impl ClientBuilder {
    pub(crate) fn new(server: &str, username: &str, password: &str) -> Self {
        Self {
            server: server.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
            session_timeout: DEFAULT_SESSION_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            accept_invalid_certs: false,
        }
    }

    /// How long a BMC session lasts before the client logs in again.
    /// Defaults to [`DEFAULT_SESSION_TIMEOUT`]; set it to match the BMC's
    /// web session timeout.
    pub fn session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }

    /// Timeout for each HTTP request. Defaults to [`DEFAULT_REQUEST_TIMEOUT`].
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Accept invalid (e.g. self-signed) TLS certificates.
    ///
    /// Supermicro BMCs ship with self-signed certificates, but this disables
    /// certificate verification entirely; only use it on trusted networks.
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Validate the configuration, returning the shared core.
    pub(crate) fn into_core(self) -> Result<Core> {
        let base = normalise_server(&self.server)?;
        Ok(Core {
            base,
            username: self.username,
            password: self.password,
            session_timeout: self.session_timeout,
            session: Mutex::new(None),
        })
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("server", &self.server)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("session_timeout", &self.session_timeout)
            .field("request_timeout", &self.request_timeout)
            .field("accept_invalid_certs", &self.accept_invalid_certs)
            .finish()
    }
}

/// An authenticated BMC session.
#[derive(Clone)]
struct Session {
    sid: String,
    expires: Instant,
}

/// State shared by the async and blocking clients.
pub(crate) struct Core {
    base: String,
    pub(crate) username: String,
    pub(crate) password: String,
    session_timeout: Duration,
    session: Mutex<Option<Session>>,
}

impl Core {
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub(crate) fn login_form(&self) -> [(&str, &str); 2] {
        [("name", &self.username), ("pwd", &self.password)]
    }

    /// The current session ID, unless there is none or it has (probably)
    /// expired.
    pub(crate) fn valid_sid(&self) -> Option<String> {
        let session = self.session.lock().unwrap_or_else(PoisonError::into_inner);
        session
            .as_ref()
            .filter(|s| Instant::now() < s.expires)
            .map(|s| s.sid.clone())
    }

    /// Record a successful login from the login response headers.
    pub(crate) fn store_session(&self, headers: &HeaderMap) -> Result<String> {
        let sid = session_id(headers).ok_or(Error::AuthenticationFailed)?;
        let mut session = self.session.lock().unwrap_or_else(PoisonError::into_inner);
        *session = Some(Session {
            sid: sid.clone(),
            expires: Instant::now() + self.session_timeout,
        });
        Ok(sid)
    }

    pub(crate) fn fmt_debug(&self, name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(name)
            .field("server", &self.base)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("session_timeout", &self.session_timeout)
            .field("logged_in", &self.valid_sid().is_some())
            .finish_non_exhaustive()
    }
}

/// Apply the TLS settings to a reqwest (async or blocking) client builder.
///
/// With `rustls`, a per-client rustls configuration using the `ring` provider
/// is supplied; otherwise `native-tls` (if enabled) handles certificates.
macro_rules! configure_tls {
    ($builder:expr, $accept_invalid_certs:expr) => {{
        let builder = $builder;
        #[cfg(feature = "rustls")]
        let builder =
            builder.tls_backend_preconfigured($crate::client::tls_config($accept_invalid_certs)?);
        #[cfg(all(feature = "native-tls", not(feature = "rustls")))]
        let builder = builder.tls_danger_accept_invalid_certs($accept_invalid_certs);
        #[cfg(not(any(feature = "rustls", feature = "native-tls")))]
        let _ = $accept_invalid_certs;
        builder
    }};
}
pub(crate) use configure_tls;

#[cfg(feature = "rustls")]
pub(crate) fn tls_config(accept_invalid_certs: bool) -> Result<rustls::ClientConfig> {
    tls::config(accept_invalid_certs)
}

pub(crate) fn cookie_header(sid: &str) -> String {
    format!("{SESSION_COOKIE}={sid}")
}

/// Unique, ordered metric kinds to fetch.
pub(crate) fn wanted(kinds: &[MetricKind]) -> impl Iterator<Item = MetricKind> + '_ {
    MetricKind::ALL.into_iter().filter(|k| kinds.contains(k))
}

/// Validate and normalise the server address to `scheme://host[:port][/path]`
/// without a trailing slash.
fn normalise_server(server: &str) -> Result<String> {
    let invalid = || Error::InvalidUrl(server.to_owned());
    let url = reqwest::Url::parse(server.trim()).map_err(|_| invalid())?;
    if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
        return Err(invalid());
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

/// Extract the session ID from `Set-Cookie` headers.
///
/// The BMC first clears the cookie (`SID=; expires=…1970…`) and then sets the
/// real one, so empty values are ignored and the last non-empty one wins.
fn session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .rev()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|cookie| {
            let pair = cookie.split(';').next()?;
            let (name, value) = pair.split_once('=')?;
            (name.trim() == SESSION_COOKIE && !value.trim().is_empty())
                .then(|| value.trim().to_owned())
        })
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    fn headers(cookies: &[&'static str]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for cookie in cookies {
            map.append(SET_COOKIE, HeaderValue::from_static(cookie));
        }
        map
    }

    #[test]
    fn session_id_skips_cleared_cookie() {
        let map = headers(&[
            "SID=; expires=Thursday,01-Jan-1970 00:00:00 GMT; HttpOnly",
            "SID=vqpcndpiuloyjnlw; path=/ ; HttpOnly",
        ]);
        assert_eq!(session_id(&map).as_deref(), Some("vqpcndpiuloyjnlw"));
    }

    #[test]
    fn session_id_absent_or_empty() {
        assert_eq!(session_id(&headers(&[])), None);
        assert_eq!(session_id(&headers(&["SID=; HttpOnly"])), None);
        assert_eq!(session_id(&headers(&["OTHER=x", "garbage"])), None);
        assert_eq!(session_id(&headers(&["SIDX=abc"])), None);
    }

    #[test]
    fn server_normalisation() {
        for (input, expected) in [
            ("http://192.168.1.1", "http://192.168.1.1"),
            ("https://bmc.example/", "https://bmc.example"),
            (" http://10.0.0.1:8080/ ", "http://10.0.0.1:8080"),
            ("https://proxy/bmc1/", "https://proxy/bmc1"),
        ] {
            assert_eq!(normalise_server(input).unwrap(), expected, "{input}");
        }
        for bad in ["192.168.1.1", "ftp://host", "", "http://"] {
            assert!(
                matches!(normalise_server(bad), Err(Error::InvalidUrl(_))),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn wanted_dedupes_and_orders() {
        use MetricKind::{Pmbus, Sensor};
        assert_eq!(
            wanted(&[Sensor, Pmbus, Sensor]).collect::<Vec<_>>(),
            [Pmbus, Sensor]
        );
        assert_eq!(wanted(&[]).count(), 0);
    }

    #[test]
    fn session_expiry() {
        let core = ClientBuilder::new("http://bmc", "u", "p")
            .session_timeout(Duration::ZERO)
            .into_core()
            .unwrap();
        assert_eq!(core.valid_sid(), None);
        core.store_session(&headers(&["SID=abc"])).unwrap();
        assert_eq!(core.valid_sid(), None, "zero timeout expires immediately");

        let core = ClientBuilder::new("http://bmc", "u", "p")
            .into_core()
            .unwrap();
        assert!(matches!(
            core.store_session(&headers(&[])),
            Err(Error::AuthenticationFailed)
        ));
        core.store_session(&headers(&["SID=abc"])).unwrap();
        assert_eq!(core.valid_sid().as_deref(), Some("abc"));
    }

    #[test]
    fn debug_redacts_password() {
        let builder = ClientBuilder::new("http://bmc", "user", "hunter2");
        let text = format!("{builder:?}");
        assert!(!text.contains("hunter2"), "{text}");
        assert!(text.contains("<redacted>"));
    }
}
