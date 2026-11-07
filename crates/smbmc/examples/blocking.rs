//! Print power supply readings using the blocking client.
//!
//! ```sh
//! SMBMC_SERVER=https://192.168.1.1 SMBMC_USER=ADMIN SMBMC_PASS=secret \
//!     cargo run --example blocking --features blocking
//! ```

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let var = |name: &str| env::var(name).map_err(|_| format!("set {name}"));
    let client = smbmc::blocking::Client::builder(
        &var("SMBMC_SERVER")?,
        &var("SMBMC_USER")?,
        &var("SMBMC_PASS")?,
    )
    .accept_invalid_certs(env::var("SMBMC_INSECURE").is_ok_and(|v| v == "1"))
    .build_blocking()?;

    for psu in client.pmbus_metrics()? {
        println!("{psu:#?}");
    }
    Ok(())
}
