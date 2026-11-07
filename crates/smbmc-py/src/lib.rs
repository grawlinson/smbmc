//! Python bindings for smbmc: the native `smbmc._smbmc` extension module.
//!
//! This layer stays deliberately thin. It returns plain dicts with raw
//! integer codes; the pure-Python `smbmc` package turns them into the
//! public `Sensor`/`PowerSupply` classes and `IntEnum`/`IntFlag` types, and
//! defines the exception hierarchy raised here.

use std::time::Duration;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use smbmc::blocking::Client as Inner;
use smbmc::{Error, PowerSupply, Sensor, SensorState};

/// Convert a Rust error into the matching `smbmc.exceptions` class.
fn to_py(py: Python<'_>, err: Error) -> PyErr {
    let name = match &err {
        Error::AuthenticationFailed => "AuthenticationError",
        Error::Http(_) => "HTTPError",
        Error::Tls(_) => "TLSError",
        Error::InvalidUrl(_) => "InvalidURLError",
        Error::UnsupportedLinearisation(_) => "UnsupportedError",
        Error::Xml(_)
        | Error::UnexpectedResponse(_)
        | Error::MissingAttribute { .. }
        | Error::InvalidHex { .. } => "ResponseError",
        _ => "SmbmcError",
    };
    let message = err.to_string();
    match py
        .import("smbmc.exceptions")
        .and_then(|module| module.getattr(name))
    {
        Ok(class) => PyErr::from_value(match class.call1((message,)) {
            Ok(instance) => instance,
            Err(e) => return e,
        }),
        Err(e) => e,
    }
}

fn sensor_dict<'py>(py: Python<'py>, s: &Sensor) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("id", s.id)?;
    d.set_item("name", &s.name)?;
    d.set_item("type", u8::from(s.sensor_type))?;
    d.set_item("unit", u8::from(s.unit))?;
    d.set_item(
        "state",
        match s.state {
            SensorState::Present => 1u8,
            SensorState::NotPresent => 2,
            SensorState::Unspecified => 0,
        },
    )?;
    d.set_item("flags", s.flags.map(|f| f.bits()))?;
    d.set_item("reading", s.reading)?;
    d.set_item("lnr", s.lnr)?;
    d.set_item("lc", s.lc)?;
    d.set_item("lnc", s.lnc)?;
    d.set_item("unc", s.unc)?;
    d.set_item("uc", s.uc)?;
    d.set_item("unr", s.unr)?;
    Ok(d)
}

fn psu_dict<'py>(py: Python<'py>, p: &PowerSupply) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("id", p.id)?;
    d.set_item("name", &p.name)?;
    d.set_item("status", p.status)?;
    d.set_item("type", p.psu_type)?;
    d.set_item("input_voltage", p.input_voltage)?;
    d.set_item("input_current", p.input_current)?;
    d.set_item("input_power", p.input_power)?;
    d.set_item("output_voltage", p.output_voltage)?;
    d.set_item("output_current", p.output_current)?;
    d.set_item("output_power", p.output_power)?;
    d.set_item("temp_1", p.temp_1)?;
    d.set_item("temp_2", p.temp_2)?;
    d.set_item("fan_1", p.fan_1)?;
    d.set_item("fan_2", p.fan_2)?;
    Ok(d)
}

fn sensor_list<'py>(py: Python<'py>, sensors: &[Sensor]) -> PyResult<Vec<Bound<'py, PyDict>>> {
    sensors.iter().map(|s| sensor_dict(py, s)).collect()
}

fn psu_list<'py>(py: Python<'py>, psus: &[PowerSupply]) -> PyResult<Vec<Bound<'py, PyDict>>> {
    psus.iter().map(|p| psu_dict(py, p)).collect()
}

fn duration(name: &str, seconds: f64) -> PyResult<Duration> {
    Duration::try_from_secs_f64(seconds)
        .map_err(|_| PyValueError::new_err(format!("{name} must be a non-negative number")))
}

/// Native BMC client. Use `smbmc.Client` instead.
#[pyclass(module = "smbmc._smbmc", frozen)]
struct Client {
    inner: Inner,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (server, username, password, session_timeout, request_timeout, verify))]
    fn new(
        py: Python<'_>,
        server: &str,
        username: &str,
        password: &str,
        session_timeout: f64,
        request_timeout: f64,
        verify: bool,
    ) -> PyResult<Self> {
        let inner = Inner::builder(server, username, password)
            .session_timeout(duration("session_timeout", session_timeout)?)
            .request_timeout(duration("request_timeout", request_timeout)?)
            .accept_invalid_certs(!verify)
            .build_blocking()
            .map_err(|e| to_py(py, e))?;
        Ok(Self { inner })
    }

    fn login(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.inner.login()).map_err(|e| to_py(py, e))
    }

    fn pmbus_metrics<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let psus = py
            .detach(|| self.inner.pmbus_metrics())
            .map_err(|e| to_py(py, e))?;
        psu_list(py, &psus)
    }

    fn sensor_metrics<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let sensors = py
            .detach(|| self.inner.sensor_metrics())
            .map_err(|e| to_py(py, e))?;
        sensor_list(py, &sensors)
    }
}

/// Parse a `SENSOR_INFO.XML` response into sensor dicts.
#[pyfunction]
fn parse_sensors<'py>(py: Python<'py>, xml: &str) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let sensors = smbmc::parse_sensors(xml).map_err(|e| to_py(py, e))?;
    sensor_list(py, &sensors)
}

/// Parse a `Get_PSInfoReadings.XML` response into power supply dicts.
#[pyfunction]
fn parse_power_supplies<'py>(py: Python<'py>, xml: &str) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let psus = smbmc::parse_power_supplies(xml).map_err(|e| to_py(py, e))?;
    psu_list(py, &psus)
}

#[pymodule]
fn _smbmc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_function(wrap_pyfunction!(parse_sensors, m)?)?;
    m.add_function(wrap_pyfunction!(parse_power_supplies, m)?)?;
    Ok(())
}
