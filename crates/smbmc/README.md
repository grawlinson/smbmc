# smbmc

[![crates.io](https://img.shields.io/crates/v/smbmc.svg)](https://crates.io/crates/smbmc)
[![docs.rs](https://docs.rs/smbmc/badge.svg)](https://docs.rs/smbmc)

An unofficial Rust interface for obtaining metrics from Supermicro BMCs via
their web interface.

- **Sensor**: temperature, fan, voltage, power supply status, etc.
- **PMBus**: power consumption, fans, temperatures, etc. per power supply.

```rust
use smbmc::{Client, MetricKind};

let client = Client::builder("https://192.168.1.1", "ADMIN", "secret")
    .accept_invalid_certs(true) // BMCs ship with self-signed certificates
    .build()?;

let metrics = client.metrics(&MetricKind::ALL).await?;
for sensor in metrics.sensor.unwrap_or_default() {
    println!("{}: {} {:?}", sensor.name, sensor.reading, sensor.unit);
}
```

The client logs in on first use and renews the session when it expires (or
when the BMC drops it). Responses can also be parsed directly with
`parse_sensors` and `parse_power_supplies`.

## Features

| Feature      | Default | Description                                         |
|--------------|---------|-----------------------------------------------------|
| `rustls`     | yes     | TLS via rustls with the `ring` provider             |
| `native-tls` |         | TLS via the platform library                        |
| `blocking`   |         | Synchronous `smbmc::blocking::Client`               |
| `serde`      |         | `Serialize`/`Deserialize` for the models            |

## Examples

```sh
SMBMC_SERVER=https://192.168.1.1 SMBMC_USER=ADMIN SMBMC_PASS=secret SMBMC_INSECURE=1 \
    cargo run --example metrics
```

## Python

The same library is available for Python as
[`smbmc` on PyPI](https://pypi.org/project/smbmc/).

## Legal

This crate is not associated with Super Micro Computer, Inc. Supermicro's
GPL-released BMC code was used as a reference, so it is licensed GPL-3.0-only.
