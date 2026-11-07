//! Parity with the Python implementation (v0.3.0).
//!
//! The golden files were produced by running smbmc 0.3.0 (pure Python) over
//! the same XML fixtures and dumping each object's `__dict__` as JSON, with
//! enums stored as their integer values.
//!
//! 0.4.0 rounds readings to three decimals where 0.3.0 truncated; none of the
//! fixture values are affected, so the results remain bit-identical.

#![allow(clippy::float_cmp)] // results must be bit-identical

use serde_json::Value;
use smbmc::{SensorState, parse_power_supplies, parse_sensors};

fn read(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn golden(name: &str) -> Vec<Value> {
    serde_json::from_str(&read(name)).unwrap()
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn int(v: &Value) -> u64 {
    v.as_u64().unwrap()
}

fn hex(v: &Value) -> u8 {
    u8::from_str_radix(v.as_str().unwrap(), 16).unwrap()
}

#[test]
fn sensors_match_python() {
    let sensors = parse_sensors(&read("ipmi_response_sensors.xml")).unwrap();
    let expected = golden("golden_sensors_v0.3.0.json");
    assert_eq!(sensors.len(), expected.len());

    for (s, e) in sensors.iter().zip(&expected) {
        let ctx = &s.name;
        assert_eq!(s.id as u64, int(&e["id"]), "{ctx}");
        assert_eq!(s.name, e["name"].as_str().unwrap());
        assert_eq!(u64::from(u8::from(s.sensor_type)), int(&e["type"]), "{ctx}");
        assert_eq!(u64::from(u8::from(s.unit)), int(&e["unit"]), "{ctx}");
        let state = match s.state {
            SensorState::Unspecified => 0,
            SensorState::Present => 1,
            SensorState::NotPresent => 2,
        };
        assert_eq!(state, int(&e["state"]), "{ctx}");
        assert_eq!(
            s.flags.map(|f| u64::from(f.bits())),
            e["flags"].as_u64(),
            "{ctx}"
        );
        for (field, actual) in [
            ("reading", s.reading),
            ("lnr", s.lnr),
            ("lc", s.lc),
            ("lnc", s.lnc),
            ("unc", s.unc),
            ("uc", s.uc),
            ("unr", s.unr),
        ] {
            assert_eq!(actual, num(&e[field]), "{ctx}.{field}");
        }
    }
}

#[test]
fn power_supplies_match_python() {
    let psus = parse_power_supplies(&read("ipmi_response_pmbus.xml")).unwrap();
    let expected = golden("golden_pmbus_v0.3.0.json");
    assert_eq!(psus.len(), expected.len());

    for (p, e) in psus.iter().zip(&expected) {
        assert_eq!(p.id as u64, int(&e["id"]));
        assert_eq!(p.name, e["name"].as_str().unwrap());
        assert_eq!(p.status, hex(&e["status"]));
        assert_eq!(p.psu_type, hex(&e["type"]));
        assert_eq!(f64::from(p.input_voltage), num(&e["input_voltage"]));
        assert_eq!(p.input_current, num(&e["input_current"]));
        assert_eq!(f64::from(p.input_power), num(&e["input_power"]));
        assert_eq!(p.output_voltage, num(&e["output_voltage"]));
        assert_eq!(p.output_current, num(&e["output_current"]));
        assert_eq!(f64::from(p.output_power), num(&e["output_power"]));
        assert_eq!(f64::from(p.temp_1), num(&e["temp_1"]));
        assert_eq!(f64::from(p.temp_2), num(&e["temp_2"]));
        assert_eq!(f64::from(p.fan_1), num(&e["fan_1"]));
        assert_eq!(f64::from(p.fan_2), num(&e["fan_2"]));
    }
}

#[cfg(feature = "serde")]
#[test]
fn models_serialize() {
    let sensors = parse_sensors(&read("ipmi_response_sensors.xml")).unwrap();
    let json = serde_json::to_value(&sensors[0]).unwrap();
    assert_eq!(json["name"], "System Temp");
    assert_eq!(json["sensor_type"], "Temperature");
    let last = serde_json::to_value(sensors.last().unwrap()).unwrap();
    assert_eq!(last["flags"], "PRESENCE_DETECTED");
}
