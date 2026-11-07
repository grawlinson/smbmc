//! Async client.

use std::fmt;

use reqwest::header::COOKIE;

use super::{
    ClientBuilder, Core, IPMI_PATH, LOGIN_PATH, MetricKind, Metrics, PMBUS_QUERY, SENSOR_QUERY,
    configure_tls, cookie_header, wanted,
};
use crate::error::{Error, Result};
use crate::models::{PowerSupply, Sensor};
use crate::{parse_power_supplies, parse_sensors};

/// Async client for a Supermicro BMC's web interface.
///
/// Sessions are managed automatically: the client logs in on first use and
/// again whenever the session timeout has elapsed. Requires a Tokio runtime.
///
/// ```no_run
/// # async fn run() -> smbmc::Result<()> {
/// let client = smbmc::Client::new("https://192.168.1.1", "ADMIN", "secret")?;
///
/// for sensor in client.sensor_metrics().await? {
///     println!("{}: {} {:?}", sensor.name, sensor.reading, sensor.unit);
/// }
/// # Ok(())
/// # }
/// ```
pub struct Client {
    core: Core,
    http: reqwest::Client,
}

impl ClientBuilder {
    /// Build an async [`Client`].
    ///
    /// # Errors
    ///
    /// [`Error::InvalidUrl`](crate::Error::InvalidUrl) if the server address
    /// is not an `http`/`https` URL, or
    /// [`Error::Http`](crate::Error::Http) if the HTTP client cannot be
    /// initialised.
    pub fn build(self) -> Result<Client> {
        let http = reqwest::Client::builder().timeout(self.request_timeout);
        let http = configure_tls!(http, self.accept_invalid_certs);
        let http = http.build()?;
        Ok(Client {
            core: self.into_core()?,
            http,
        })
    }
}

impl Client {
    /// Create a client with default settings.
    ///
    /// `server` is the BMC's base address, e.g. `"https://192.168.1.1"`.
    ///
    /// # Errors
    ///
    /// See [`ClientBuilder::build`].
    pub fn new(server: &str, username: &str, password: &str) -> Result<Self> {
        Self::builder(server, username, password).build()
    }

    /// Start configuring a client.
    pub fn builder(server: &str, username: &str, password: &str) -> ClientBuilder {
        ClientBuilder::new(server, username, password)
    }

    /// Log in, replacing any existing session.
    ///
    /// Calling this is optional; other methods log in as needed.
    ///
    /// # Errors
    ///
    /// [`Error::AuthenticationFailed`](crate::Error::AuthenticationFailed) if
    /// the BMC does not issue a session ID, or
    /// [`Error::Http`](crate::Error::Http) on transport errors and non-2xx
    /// responses.
    pub async fn login(&self) -> Result<()> {
        self.new_session().await.map(drop)
    }

    /// Fetch every power supply on the PMBus interface.
    ///
    /// # Errors
    ///
    /// Login, HTTP or parsing errors; see [`crate::Error`].
    pub async fn pmbus_metrics(&self) -> Result<Vec<PowerSupply>> {
        self.fetch(PMBUS_QUERY, parse_power_supplies).await
    }

    /// Fetch every sensor known to the BMC.
    ///
    /// # Errors
    ///
    /// Login, HTTP or parsing errors; see [`crate::Error`].
    pub async fn sensor_metrics(&self) -> Result<Vec<Sensor>> {
        self.fetch(SENSOR_QUERY, parse_sensors).await
    }

    /// Fetch the requested kinds of metrics (duplicates are ignored).
    /// Use [`MetricKind::ALL`] for everything.
    ///
    /// # Errors
    ///
    /// The first error encountered; see [`crate::Error`].
    pub async fn metrics(&self, kinds: &[MetricKind]) -> Result<Metrics> {
        let mut metrics = Metrics::default();
        for kind in wanted(kinds) {
            match kind {
                MetricKind::Pmbus => metrics.pmbus = Some(self.pmbus_metrics().await?),
                MetricKind::Sensor => metrics.sensor = Some(self.sensor_metrics().await?),
            }
        }
        Ok(metrics)
    }

    async fn new_session(&self) -> Result<String> {
        let response = self
            .http
            .post(self.core.url(LOGIN_PATH))
            .form(&self.core.login_form())
            .send()
            .await?
            .error_for_status()?;
        self.core.store_session(response.headers())
    }

    /// Run a query and parse the response.
    ///
    /// If an existing session was used and the response is not XML (the BMC
    /// returns its login page once a session is dropped, e.g. after a BMC
    /// reboot), log in again and retry once.
    async fn fetch<T>(&self, field: (&str, &str), parse: fn(&str) -> Result<T>) -> Result<T> {
        let (sid, fresh) = match self.core.valid_sid() {
            Some(sid) => (sid, false),
            None => (self.new_session().await?, true),
        };
        match parse(&self.post_query(&sid, field).await?) {
            Err(Error::Xml(_) | Error::UnexpectedResponse(_)) if !fresh => {
                let sid = self.new_session().await?;
                parse(&self.post_query(&sid, field).await?)
            }
            result => result,
        }
    }

    async fn post_query(&self, sid: &str, field: (&str, &str)) -> Result<String> {
        Ok(self
            .http
            .post(self.core.url(IPMI_PATH))
            .header(COOKIE, cookie_header(sid))
            .form(&[field])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.core.fmt_debug("Client", f)
    }
}
