//! An unofficial interface for obtaining metrics from Supermicro BMCs.
//!
//! The following metrics are accessible:
//!
//! - **Sensor**: temperature, fan, voltage, power supply status, etc.
//! - **PMBus**: power consumption, fans, temperatures, etc. per power supply.
//!
//! This library depends on the BMC web interface being available.
//!
//! # Clients
//!
//! [`Client`] is async (Tokio); enable the `blocking` feature for
//! [`blocking::Client`]. Both log in on first use and refresh the session
//! when it expires.
//!
//! ```no_run
//! # async fn run() -> smbmc::Result<()> {
//! use smbmc::{Client, MetricKind};
//!
//! let client = Client::new("https://192.168.1.1", "ADMIN", "secret")?;
//! let metrics = client.metrics(&MetricKind::ALL).await?;
//! println!("{:#?}", metrics.pmbus);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `rustls` *(default)*: TLS via rustls.
//! - `native-tls`: TLS via the platform library instead.
//! - `blocking`: the synchronous [`blocking::Client`].
//! - `serde`: `Serialize`/`Deserialize` for the models.
//!
//! # Parsing only
//!
//! The parsing layer is independent of any HTTP client, so responses obtained
//! by other means can be decoded with [`parse_sensors`] and
//! [`parse_power_supplies`]:
//!
//! ```
//! let xml = r#"<?xml version="1.0"?>
//! <IPMI><SENSOR_INFO>
//!   <SENSOR ID="001" NAME="System Temp" READING="19c000" OPTION="c0"
//!     UNR="5a" UC="55" UNC="50" LNC="fb" LC="f9" LNR="f7" STYPE="01"
//!     RTYPE="01" ERTYPE="01" UNIT1="80" UNIT="01" L="00" M="0100" B="0000" RB="00"/>
//! </SENSOR_INFO></IPMI>"#;
//!
//! let sensors = smbmc::parse_sensors(xml)?;
//! assert_eq!(sensors[0].name, "System Temp");
//! assert_eq!(sensors[0].reading, 25.0);
//! assert_eq!(sensors[0].unit, smbmc::SensorUnit::DegreesCelsius);
//! # Ok::<(), smbmc::Error>(())
//! ```
//!
//! Supermicro have released some BMC/IPMI code under the GPL, which has been
//! used as a reference; this crate is therefore licensed as GPL-3.0-only. It
//! is not associated with Super Micro Computer, Inc.

mod client;
mod convert;
mod error;
mod models;
mod pmbus;
mod sensor;
mod xml;

pub use client::async_impl::Client;
pub use client::{
    ClientBuilder, DEFAULT_REQUEST_TIMEOUT, DEFAULT_SESSION_TIMEOUT, MetricKind, Metrics,
};
pub use error::{Error, Result};
pub use models::{PowerSupply, PowerSupplyFlag, Sensor, SensorState, SensorType, SensorUnit};
pub use pmbus::parse_power_supplies;
pub use sensor::parse_sensors;

/// Synchronous client (requires the `blocking` feature).
#[cfg(feature = "blocking")]
pub mod blocking {
    pub use crate::client::blocking::Client;
}
