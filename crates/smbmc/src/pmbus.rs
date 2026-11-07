//! PMBus power supply decoding (port of `ipmi_pmbus.py`).

use crate::convert::{parse_hex, parse_hex_u8};
use crate::error::Result;
use crate::models::PowerSupply;
use crate::xml::{self, Attrs};

/// Parse a `Get_PSInfoReadings.XML` response into power supplies.
///
/// Each power supply's `id` is its slot index within the response.
///
/// # Errors
///
/// Returns an error if the XML is malformed or a required attribute is
/// missing or not valid hexadecimal.
pub fn parse_power_supplies(xml: &str) -> Result<Vec<PowerSupply>> {
    xml::extract(xml, "PSItem")?
        .iter()
        .enumerate()
        .map(|(id, item)| process_psu(item).map(|psu| PowerSupply { id, ..psu }))
        .collect()
}

fn process_psu(item: &Attrs) -> Result<PowerSupply> {
    let hex = |attribute: &'static str| item.get(attribute).and_then(parse_hex);
    let scaled =
        |attribute: &'static str, divisor: f64| hex(attribute).map(|v| f64::from(v) / divisor);

    Ok(PowerSupply {
        id: 0,
        name: item.get("name")?.to_owned(),
        status: parse_hex_u8(item.get("a_b_PS_Status_I2C")?)?,
        psu_type: parse_hex_u8(item.get("psType")?)?,
        input_voltage: hex("acInVoltage")?,
        input_current: scaled("acInCurrent", 1000.0)?,
        input_power: hex("acInPower")?,
        output_voltage: scaled("dc12OutVoltage", 10.0)?,
        output_current: scaled("dc12OutCurrent", 1000.0)?,
        output_power: hex("dcOutPower")?,
        temp_1: hex("temp1")?,
        temp_2: hex("temp2")?,
        fan_1: hex("fan1")?,
        fan_2: hex("fan2")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}.xml", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    #[test]
    fn parse_power_supplies_fixture() {
        let psus = parse_power_supplies(&fixture("ipmi_response_pmbus")).unwrap();
        assert_eq!(psus.len(), 4);

        let populated = PowerSupply {
            id: 1,
            name: "PSU0SERIAL0NO00".into(),
            status: 0x01,
            psu_type: 0x01,
            input_voltage: 239,
            input_current: 0.359,
            input_power: 84,
            output_voltage: 12.1,
            output_current: 5.75,
            output_power: 69,
            temp_1: 40,
            temp_2: 55,
            fan_1: 2894,
            fan_2: 3847,
        };
        for psu in &psus {
            if psu.id == 1 {
                assert_eq!(psu, &populated);
            } else {
                let empty = PowerSupply {
                    id: psu.id,
                    status: 0xff,
                    ..PowerSupply::default()
                };
                assert_eq!(psu, &empty);
            }
        }
    }

    #[test]
    fn invalid_hex_is_reported() {
        let xml =
            r#"<IPMI><PSItem name="" a_b_PS_Status_I2C="1" psType="1" acInVoltage="zz"/></IPMI>"#;
        assert!(matches!(
            parse_power_supplies(xml),
            Err(Error::InvalidHex { .. })
        ));
    }
}
