//! Low-level numeric conversions for raw IPMI values (port of `util.py`).

use crate::error::{Error, Result};

/// Parse a hexadecimal string, accepting an optional `0x`/`0X` prefix and
/// either letter case (mirrors Python's `int(value, 16)` for our inputs).
pub(crate) fn parse_hex(value: &str) -> Result<u32> {
    let trimmed = value.trim();
    let digits = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u32::from_str_radix(digits, 16).map_err(|_| Error::InvalidHex {
        value: value.to_owned(),
    })
}

/// Parse a hexadecimal string that must fit in a single byte.
pub(crate) fn parse_hex_u8(value: &str) -> Result<u8> {
    u8::try_from(parse_hex(value)?).map_err(|_| Error::InvalidHex {
        value: value.to_owned(),
    })
}

/// Interpret the low `bits` bits of `value` as a two's complement integer.
///
/// `bits == 0` returns `value` unchanged.
pub(crate) fn signed_int(value: u32, bits: u32) -> i64 {
    if bits == 0 {
        return i64::from(value);
    }
    let modulus = 1i64 << bits;
    let v = i64::from(value) % modulus;
    if v < modulus / 2 { v } else { v - modulus }
}

/// Decode a 10-bit value stored as two bytes: the first byte holds the least
/// significant 8 bits and the top two bits of the second byte hold the most
/// significant 2 bits (IPMI 2.0, §43.1, `M` and `B` fields).
pub(crate) fn ten_bit(value: u32) -> u32 {
    ((value & 0xC0) << 2) + (value >> 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_int_cases() {
        for (unsigned, bits, signed) in [(129, 8, -127), (874, 10, -150), (200, 0, 200), (7, 4, 7)]
        {
            assert_eq!(
                signed_int(unsigned, bits),
                signed,
                "signed_int({unsigned}, {bits})"
            );
        }
    }

    // Ported from `test_hex_signed_int`.
    #[test]
    fn hex_signed_int_cases() {
        for (hex, bits, expected) in [("10", 0, 0x10), ("120", 8, 0x20)] {
            assert_eq!(signed_int(parse_hex(hex).unwrap(), bits), expected);
        }
        assert_eq!(signed_int(parse_hex("f7").unwrap(), 8), -9);
    }

    #[test]
    fn ten_bit_cases() {
        for (hex, expected) in [
            ("FF", 768),
            ("AB", 512),
            ("200", 2),
            ("0x200", 2),
            ("4000", 64),
        ] {
            assert_eq!(ten_bit(parse_hex(hex).unwrap()), expected, "ten_bit({hex})");
        }
    }

    #[test]
    fn parse_hex_accepts_prefix_and_case() {
        assert_eq!(parse_hex("0xFf").unwrap(), 255);
        assert_eq!(parse_hex("0Xff").unwrap(), 255);
        assert_eq!(parse_hex("ff").unwrap(), 255);
    }

    #[test]
    fn parse_hex_rejects_garbage() {
        for bad in ["", "0x", "zz", "-1"] {
            assert!(
                matches!(parse_hex(bad), Err(Error::InvalidHex { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn parse_hex_u8_range() {
        assert_eq!(parse_hex_u8("ff").unwrap(), 0xff);
        assert!(matches!(parse_hex_u8("100"), Err(Error::InvalidHex { .. })));
    }
}
