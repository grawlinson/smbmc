//! IPMI sensor decoding (port of `ipmi_sensor.py`).

use crate::convert::{parse_hex, parse_hex_u8, signed_int, ten_bit};
use crate::error::{Error, Result};
use crate::models::{PowerSupplyFlag, Sensor, SensorState, SensorType, SensorUnit};
use crate::xml::{self, Attrs};

/// Readings are rounded to this many fractional units (i.e. 3 decimal places).
const SENSOR_READING_SCALE: f64 = 1000.0;

/// Linearisation formula code for `y = x` (IPMI 2.0, §43.1, byte 24).
const LINEARISATION_LINEAR: u8 = 0x00;

/// Event/Reading Type code for threshold sensors (IPMI 2.0, table 42-1).
const ER_TYPE_THRESHOLD: u32 = 0x01;

/// Parse a `SENSOR_INFO.XML` response into sensors.
///
/// Each sensor's `id` is its index within the response.
///
/// # Errors
///
/// Returns an error if the XML is malformed, a required attribute is missing
/// or not valid hexadecimal, a sensor uses an unimplemented linearisation
/// formula.
///
/// Discrete sensors other than power supplies are returned with
/// `flags: None`, as their state decoding is not implemented yet.
pub fn parse_sensors(xml: &str) -> Result<Vec<Sensor>> {
    xml::extract(xml, "SENSOR")?
        .iter()
        .enumerate()
        .map(|(id, item)| process_sensor(item).map(|sensor| Sensor { id, ..sensor }))
        .collect()
}

fn process_sensor(item: &Attrs) -> Result<Sensor> {
    if is_threshold_sensor(item.get("ERTYPE")?)? {
        process_threshold_sensor(item)
    } else {
        process_discrete_sensor(item)
    }
}

/// Sensor Reading Conversion Formula, pre-linearisation
/// (IPMI 2.0, §36.3): `y = (M·x + B·10^K1) · 10^K2`.
///
/// `m` and `b` are the raw two-byte SDR fields (§43.1, bytes 25–28) and `rb`
/// packs the result exponent `K2` (upper nibble) and `B` exponent `K1`
/// (lower nibble).
fn reading_conversion(data: i64, m: &str, b: &str, rb: &str) -> Result<f64> {
    let rb = parse_hex(rb)?;
    let m = signed_int(ten_bit(parse_hex(m)?), 10);
    let b = signed_int(ten_bit(parse_hex(b)?), 10);
    let k2 = signed_int(rb >> 4, 4);
    let k1 = signed_int(rb & 0x0F, 4);

    // `powf` (libm `pow`) keeps results bit-identical to Python's `math.pow`.
    #[allow(clippy::cast_precision_loss)]
    let value = (m as f64 * data as f64 + b as f64 * 10f64.powf(k1 as f64)) * 10f64.powf(k2 as f64);
    Ok(value)
}

/// Threshold vs discrete, from the Event/Reading Type code (SDR byte 14).
fn is_threshold_sensor(er_type: &str) -> Result<bool> {
    Ok(parse_hex(er_type)? == ER_TYPE_THRESHOLD)
}

/// Bits 7:6 of Sensor Units 1 equal to `10b` mean readings are two's
/// complement.
fn is_analog_data_format(unit_type_1: &str) -> Result<bool> {
    Ok(parse_hex(unit_type_1)? >> 6 == 2)
}

/// Best guess at presence from the `OPTION` byte.
fn get_sensor_state(option: &str) -> Result<SensorState> {
    Ok(if parse_hex(option)? & 0x40 != 0 {
        SensorState::Present
    } else {
        SensorState::NotPresent
    })
}

/// Apply the linearisation formula, rounding to three decimal places.
///
/// Python 0.3.0 truncated instead, which occasionally dropped 0.001 when
/// floating-point error landed just below the true value (e.g. 4.055 is
/// `4054.9999999999995` after scaling).
fn perform_linearisation(method: u8, reading: f64) -> Result<f64> {
    match method {
        LINEARISATION_LINEAR => Ok((reading * SENSOR_READING_SCALE).round() / SENSOR_READING_SCALE),
        other => Err(Error::UnsupportedLinearisation(other)),
    }
}

fn process_threshold_sensor(item: &Attrs) -> Result<Sensor> {
    let analog = is_analog_data_format(item.get("UNIT1")?)?;
    let (m, b, rb) = (item.get("M")?, item.get("B")?, item.get("RB")?);
    let method = parse_hex_u8(item.get("L")?)?;

    let value = |raw: &str| -> Result<f64> {
        let raw = parse_hex(raw)?;
        let data = if analog {
            signed_int(raw, 8)
        } else {
            i64::from(raw)
        };
        perform_linearisation(method, reading_conversion(data, m, b, rb)?)
    };

    let reading = item.get("READING")?;
    // The first byte of READING is the raw sensor value.
    let raw_reading = reading.get(..2).unwrap_or(reading);

    Ok(Sensor {
        name: item.get("NAME")?.to_owned(),
        sensor_type: SensorType::from(parse_hex_u8(item.get("STYPE")?)?),
        unit: SensorUnit::from(parse_hex_u8(item.get("UNIT")?)?),
        state: get_sensor_state(item.get("OPTION")?)?,
        reading: value(raw_reading)?,
        lnr: value(item.get("LNR")?)?,
        lc: value(item.get("LC")?)?,
        lnc: value(item.get("LNC")?)?,
        unc: value(item.get("UNC")?)?,
        uc: value(item.get("UC")?)?,
        unr: value(item.get("UNR")?)?,
        ..Sensor::default()
    })
}

fn process_discrete_sensor(item: &Attrs) -> Result<Sensor> {
    let sensor_type = SensorType::from(parse_hex_u8(item.get("STYPE")?)?);
    let state = get_sensor_state(item.get("OPTION")?)?;
    let mut sensor = Sensor {
        name: item.get("NAME")?.to_owned(),
        sensor_type,
        state,
        ..Sensor::default()
    };

    // Python returned `None` here, which crashed `process_sensor_response`.
    if state == SensorState::NotPresent {
        return Ok(sensor);
    }

    // Only power supply state decoding is implemented; other discrete
    // sensors are returned without flags rather than failing the whole call.
    if sensor_type == SensorType::PowerSupply {
        // The second byte of READING holds the state bits.
        let reading = item.get("READING")?;
        let state_bits = parse_hex_u8(reading.get(2..4).unwrap_or_default())?;
        sensor.flags = Some(PowerSupplyFlag::from_bits_retain(state_bits));
    }
    Ok(sensor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::attrs;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}.xml", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    #[test]
    #[allow(clippy::float_cmp)] // exact parity with the Python results is the point
    fn reading_conversion_cases() {
        for (data, expected) in [
            ("84", 13.200_000_000_000_001),
            ("83", 13.1),
            ("7f", 12.700_000_000_000_001),
            ("6a", 10.6),
            ("65", 10.1),
            ("64", 10.0),
        ] {
            let data = i64::from(parse_hex(data).unwrap());
            assert_eq!(
                reading_conversion(data, "6400", "0000", "d0").unwrap(),
                expected
            );
        }
    }

    #[test]
    fn is_threshold_sensor_cases() {
        for (er_type, expected) in [
            ("0x01", true),
            ("1", true),
            ("FF", false),
            ("2", false),
            ("0x02", false),
        ] {
            assert_eq!(is_threshold_sensor(er_type).unwrap(), expected, "{er_type}");
        }
    }

    #[test]
    fn is_analog_data_format_cases() {
        for (unit1, expected) in [
            ("0x80", true),
            ("91", true),
            ("0xC0", false),
            ("C0", false),
            ("0x00", false),
            ("00", false),
        ] {
            assert_eq!(is_analog_data_format(unit1).unwrap(), expected, "{unit1}");
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn conversion_then_rounding_keeps_exact_value() {
        // 5VCC with raw 0x96: (27·150 + 5·10) · 10⁻³ = 4.055 exactly.
        let raw = reading_conversion(0x96, "1b00", "0500", "d0").unwrap();
        assert_eq!(perform_linearisation(0, raw).unwrap(), 4.055);
        // 3.3VCC with raw 0xeb: (17·235 + 52·10) · 10⁻³ = 4.047 exactly.
        let raw = reading_conversion(0xeb, "1100", "3400", "d0").unwrap();
        assert_eq!(perform_linearisation(0, raw).unwrap(), 4.047);
    }

    #[test]
    fn get_sensor_state_cases() {
        assert_eq!(get_sensor_state("0x40").unwrap(), SensorState::Present);
        assert_eq!(get_sensor_state("0x80").unwrap(), SensorState::NotPresent);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn perform_linearisation_cases() {
        for (reading, expected) in [
            (5.000_000_1, 5.0),
            (5.0, 5.0),
            (5.7, 5.7),
            (5.600_000_000_000_000_5, 5.6),
            (13.200_000_000_000_001, 13.2),
            (13.1, 13.1),
            (12.700_000_000_000_001, 12.7),
            (0.0, 0.0),
            // Truncation (Python 0.3.0) produced 4.054 / 4.046 / -4.054 here.
            (4.055, 4.055),
            (4.047, 4.047),
            (-4.055, -4.055),
            (4.0555, 4.056),
        ] {
            assert_eq!(
                perform_linearisation(0, reading).unwrap(),
                expected,
                "{reading}"
            );
        }
    }

    #[test]
    fn perform_linearisation_unsupported() {
        assert!(matches!(
            perform_linearisation(2, 2.0),
            Err(Error::UnsupportedLinearisation(2))
        ));
    }

    #[test]
    fn discrete_sensor_unsupported_type_has_no_flags() {
        let item = attrs(
            "SENSOR",
            &[
                ("NAME", "x"),
                ("STYPE", "01"),
                ("READING", "010100"),
                ("OPTION", "c0"),
            ],
        );
        let sensor = process_discrete_sensor(&item).unwrap();
        assert_eq!(sensor.state, SensorState::Present);
        assert_eq!(sensor.sensor_type, SensorType::Temperature);
        assert_eq!(sensor.flags, None);
    }

    #[test]
    fn discrete_sensor_not_present() {
        let item = attrs(
            "SENSOR",
            &[
                ("NAME", "x"),
                ("STYPE", "01"),
                ("READING", "010100"),
                ("OPTION", "00"),
            ],
        );
        let sensor = process_discrete_sensor(&item).unwrap();
        assert_eq!(sensor.state, SensorState::NotPresent);
        assert_eq!(sensor.sensor_type, SensorType::Temperature);
        assert_eq!(sensor.flags, None);
    }

    #[test]
    fn discrete_power_supply_short_reading() {
        let item = attrs(
            "SENSOR",
            &[
                ("NAME", "PS"),
                ("STYPE", "08"),
                ("READING", "01"),
                ("OPTION", "c0"),
            ],
        );
        assert!(matches!(
            process_discrete_sensor(&item),
            Err(Error::InvalidHex { .. })
        ));
    }

    #[test]
    fn parse_sensors_fixture() {
        let sensors = parse_sensors(&fixture("ipmi_response_sensors")).unwrap();
        assert_eq!(sensors.len(), 28);
        for (i, sensor) in sensors.iter().enumerate() {
            assert_eq!(sensor.id, i);
        }
    }

    #[test]
    fn parse_sensors_missing_attribute() {
        let err = parse_sensors(r#"<IPMI><SENSOR NAME="x"/></IPMI>"#).unwrap_err();
        assert!(matches!(
            err,
            Error::MissingAttribute {
                attribute: "ERTYPE",
                ..
            }
        ));
    }
}
