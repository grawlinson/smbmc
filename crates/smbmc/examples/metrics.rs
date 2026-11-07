//! Print all metrics from a BMC.
//!
//! ```sh
//! SMBMC_SERVER=https://192.168.1.1 SMBMC_USER=ADMIN SMBMC_PASS=secret \
//!     cargo run --example metrics
//! ```
//!
//! Set `SMBMC_INSECURE=1` to accept the BMC's self-signed certificate.

use std::env;

use smbmc::{Client, MetricKind, SensorState};

fn var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| {
        eprintln!("error: set {name} (see the example's documentation)");
        std::process::exit(2);
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder(&var("SMBMC_SERVER"), &var("SMBMC_USER"), &var("SMBMC_PASS"))
        .accept_invalid_certs(env::var("SMBMC_INSECURE").is_ok_and(|v| v == "1"))
        .build()?;

    let metrics = client.metrics(&MetricKind::ALL).await?;

    println!(
        "{:<20} {:>12}  {:<16} {:>10} {:>10}",
        "SENSOR", "READING", "UNIT", "LC", "UC"
    );
    for s in metrics.sensor.unwrap_or_default() {
        if s.state == SensorState::NotPresent {
            continue;
        }
        match s.flags {
            Some(flags) => println!("{:<20} {flags:?}", s.name),
            None => println!(
                "{:<20} {:>12.3}  {:<16} {:>10.3} {:>10.3}",
                s.name,
                s.reading,
                format!("{:?}", s.unit),
                s.lc,
                s.uc
            ),
        }
    }

    println!();
    for p in metrics.pmbus.unwrap_or_default() {
        if p.name.is_empty() {
            println!("PSU {}: not present", p.id);
        } else {
            println!(
                "PSU {} ({}): {} V / {} W in, {} V / {} W out, {}/{} °C, {}/{} rpm",
                p.id,
                p.name,
                p.input_voltage,
                p.input_power,
                p.output_voltage,
                p.output_power,
                p.temp_1,
                p.temp_2,
                p.fan_1,
                p.fan_2
            );
        }
    }
    Ok(())
}
