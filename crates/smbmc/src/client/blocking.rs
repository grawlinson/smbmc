//! Blocking client.

use std::fmt;

use reqwest::header::COOKIE;

use super::{
    ClientBuilder, Core, IPMI_PATH, LOGIN_PATH, MetricKind, Metrics, PMBUS_QUERY, SENSOR_QUERY,
    configure_tls, cookie_header, wanted,
};
use crate::error::{Error, Result};
use crate::models::{PowerSupply, Sensor};
use crate::{parse_power_supplies, parse_sensors};

/// Blocking client for a Supermicro BMC's web interface.
///
/// Identical to the async [`crate::Client`], but each method blocks the
/// current thread. It must not be used from within an async runtime.
///
/// ```no_run
/// # fn run() -> smbmc::Result<()> {
/// let client = smbmc::blocking::Client::new("https://192.168.1.1", "ADMIN", "secret")?;
///
/// for psu in client.pmbus_metrics()? {
///     println!("PSU {}: {} W in, {} W out", psu.id, psu.input_power, psu.output_power);
/// }
/// # Ok(())
/// # }
/// ```
pub struct Client {
    core: Core,
    http: reqwest::blocking::Client,
}

impl ClientBuilder {
    /// Build a [`blocking::Client`](Client).
    ///
    /// # Errors
    ///
    /// See [`ClientBuilder::build`].
    pub fn build_blocking(self) -> Result<Client> {
        let http = reqwest::blocking::Client::builder().timeout(self.request_timeout);
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
    /// # Errors
    ///
    /// See [`ClientBuilder::build`].
    pub fn new(server: &str, username: &str, password: &str) -> Result<Self> {
        Self::builder(server, username, password).build_blocking()
    }

    /// Start configuring a client; finish with
    /// [`ClientBuilder::build_blocking`].
    pub fn builder(server: &str, username: &str, password: &str) -> ClientBuilder {
        ClientBuilder::new(server, username, password)
    }

    /// Log in, replacing any existing session. See [`crate::Client::login`].
    ///
    /// # Errors
    ///
    /// See [`crate::Client::login`].
    pub fn login(&self) -> Result<()> {
        self.new_session().map(drop)
    }

    /// Fetch every power supply on the PMBus interface.
    ///
    /// # Errors
    ///
    /// Login, HTTP or parsing errors; see [`crate::Error`].
    pub fn pmbus_metrics(&self) -> Result<Vec<PowerSupply>> {
        self.fetch(PMBUS_QUERY, parse_power_supplies)
    }

    /// Fetch every sensor known to the BMC.
    ///
    /// # Errors
    ///
    /// Login, HTTP or parsing errors; see [`crate::Error`].
    pub fn sensor_metrics(&self) -> Result<Vec<Sensor>> {
        self.fetch(SENSOR_QUERY, parse_sensors)
    }

    /// Fetch the requested kinds of metrics (duplicates are ignored).
    ///
    /// # Errors
    ///
    /// The first error encountered; see [`crate::Error`].
    pub fn metrics(&self, kinds: &[MetricKind]) -> Result<Metrics> {
        let mut metrics = Metrics::default();
        for kind in wanted(kinds) {
            match kind {
                MetricKind::Pmbus => metrics.pmbus = Some(self.pmbus_metrics()?),
                MetricKind::Sensor => metrics.sensor = Some(self.sensor_metrics()?),
            }
        }
        Ok(metrics)
    }

    fn new_session(&self) -> Result<String> {
        let response = self
            .http
            .post(self.core.url(LOGIN_PATH))
            .form(&self.core.login_form())
            .send()?
            .error_for_status()?;
        self.core.store_session(response.headers())
    }

    /// Run a query and parse the response.
    ///
    /// If an existing session was used and the response is not XML (the BMC
    /// returns its login page once a session is dropped, e.g. after a BMC
    /// reboot), log in again and retry once.
    fn fetch<T>(&self, field: (&str, &str), parse: fn(&str) -> Result<T>) -> Result<T> {
        let (sid, fresh) = match self.core.valid_sid() {
            Some(sid) => (sid, false),
            None => (self.new_session()?, true),
        };
        match parse(&self.post_query(&sid, field)?) {
            Err(Error::Xml(_) | Error::UnexpectedResponse(_)) if !fresh => {
                let sid = self.new_session()?;
                parse(&self.post_query(&sid, field)?)
            }
            result => result,
        }
    }

    fn post_query(&self, sid: &str, field: (&str, &str)) -> Result<String> {
        Ok(self
            .http
            .post(self.core.url(IPMI_PATH))
            .header(COOKIE, cookie_header(sid))
            .form(&[field])
            .send()?
            .error_for_status()?
            .text()?)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.core.fmt_debug("blocking::Client", f)
    }
}
