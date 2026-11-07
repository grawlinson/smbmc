//! Public data models (port of `models.py`).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

macro_rules! code_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $code:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[non_exhaustive]
        pub enum $name {
            /// Not specified by the BMC.
            #[default]
            Unspecified,
            $( $(#[$vmeta])* $variant, )+
            /// A raw code this library does not (yet) recognise.
            Other(u8),
        }

        impl From<u8> for $name {
            fn from(code: u8) -> Self {
                match code {
                    0 => Self::Unspecified,
                    $( $code => Self::$variant, )+
                    other => Self::Other(other),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                match value {
                    $name::Unspecified => 0,
                    $( $name::$variant => $code, )+
                    $name::Other(code) => code,
                }
            }
        }
    };
}

code_enum! {
    /// Sensor type (IPMI 2.0, table 42-3).
    pub enum SensorType {
        /// Temperature.
        Temperature = 1,
        /// Voltage.
        Voltage = 2,
        /// Fan.
        Fan = 4,
        /// Power supply.
        PowerSupply = 8,
    }
}

code_enum! {
    /// Unit of a sensor reading (IPMI 2.0, table 43-15).
    pub enum SensorUnit {
        /// Degrees Celsius (°C).
        DegreesCelsius = 1,
        /// Volts (V).
        Volts = 4,
        /// Amps (A).
        Amps = 5,
        /// Watts (W).
        Watts = 6,
        /// Revolutions per minute (rpm).
        Rpm = 18,
    }
}

/// Whether a sensor is present, as guessed from the `OPTION` byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SensorState {
    /// Unknown.
    #[default]
    Unspecified = 0,
    /// The sensor is present.
    Present = 1,
    /// The sensor is not present; its readings are meaningless.
    NotPresent = 2,
}

bitflags::bitflags! {
    /// Power supply specific discrete sensor events (IPMI 2.0, table 42-3,
    /// sensor type 08h).
    ///
    /// Unknown bits are retained.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct PowerSupplyFlag: u8 {
        /// Presence detected.
        const PRESENCE_DETECTED = 1;
        /// Power supply failure detected.
        const FAILURE = 1 << 1;
        /// Predictive failure.
        const PREDICTIVE_FAILURE = 1 << 2;
        /// Power supply input lost (AC/DC).
        const SOURCE_INPUT_LOST = 1 << 3;
        /// Power supply input lost or out of range.
        const SOURCE_INPUT_OUT_OF_RANGE = 1 << 4;
        /// Power supply input out of range, but present.
        const SOURCE_INPUT_DETECTED_OUT_OF_RANGE = 1 << 5;
        /// Configuration error.
        const CONFIGURATION_ERROR = 1 << 6;
        /// Power supply inactive (standby).
        const STANDBY = 1 << 7;
    }
}

/// A sensor reported by the BMC.
///
/// For discrete sensors, `reading` and the thresholds are `0.0`; see `flags`
/// instead.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Sensor {
    /// Index of the sensor within the response (pseudo-unique).
    pub id: usize,
    /// Sensor name.
    pub name: String,
    /// Sensor type.
    pub sensor_type: SensorType,
    /// Reading unit.
    pub unit: SensorUnit,
    /// Sensor state.
    pub state: SensorState,
    /// Discrete power supply sensors only: event flags.
    pub flags: Option<PowerSupplyFlag>,
    /// Sensor reading.
    pub reading: f64,
    /// Lower non-recoverable threshold.
    pub lnr: f64,
    /// Lower critical threshold.
    pub lc: f64,
    /// Lower non-critical threshold.
    pub lnc: f64,
    /// Upper non-critical threshold.
    pub unc: f64,
    /// Upper critical threshold.
    pub uc: f64,
    /// Upper non-recoverable threshold.
    pub unr: f64,
}

/// A power supply reported over PMBus.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct PowerSupply {
    /// Slot index in the chassis.
    pub id: usize,
    /// Serial number (empty when the slot is unpopulated).
    pub name: String,
    /// Raw `a_b_PS_Status_I2C` byte; possibly the I2C status
    /// (`0x01` enabled, `0xff` absent).
    pub status: u8,
    /// Raw `psType` byte; possibly the power state (`1` on, `0` off/standby).
    pub psu_type: u8,
    /// Input voltage (V AC).
    pub input_voltage: u32,
    /// Input current (A).
    pub input_current: f64,
    /// Input power (W).
    pub input_power: u32,
    /// Output voltage (V DC).
    pub output_voltage: f64,
    /// Output current (A).
    pub output_current: f64,
    /// Output power (W).
    pub output_power: u32,
    /// Temperature 1, possibly intake (°C).
    pub temp_1: u32,
    /// Temperature 2, possibly outlet (°C).
    pub temp_2: u32,
    /// Fan 1 (rpm).
    pub fan_1: u32,
    /// Fan 2 (rpm).
    pub fan_2: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_enums_round_trip() {
        for code in 0..=u8::MAX {
            assert_eq!(u8::from(SensorType::from(code)), code);
            assert_eq!(u8::from(SensorUnit::from(code)), code);
        }
        assert_eq!(SensorType::from(8), SensorType::PowerSupply);
        assert_eq!(SensorType::from(7), SensorType::Other(7));
        assert_eq!(SensorUnit::from(18), SensorUnit::Rpm);
        assert_eq!(SensorUnit::default(), SensorUnit::Unspecified);
    }

    #[test]
    fn power_supply_flag_retains_unknown_bits() {
        let flags = PowerSupplyFlag::from_bits_retain(0b1000_0011);
        assert!(flags.contains(PowerSupplyFlag::PRESENCE_DETECTED | PowerSupplyFlag::FAILURE));
        assert!(flags.contains(PowerSupplyFlag::STANDBY));
        assert_eq!(flags.bits(), 0b1000_0011);
    }
}
