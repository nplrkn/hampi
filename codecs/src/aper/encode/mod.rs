//! ASN.1 Aper Encoder module.

use crate::aper::AperCodecData;
use crate::aper::AperCodecError;
use bitvec::prelude::*;
mod encode_internal;
use bitvec::view::AsBits;
use encode_internal::*;

/// Encode a Choice Index
///
/// During Encoding a 'CHOICE' Type to help decoding, the 'CHOICE' Index is encoded first, followed
/// by the actual encoding of the 'CHOICE' variant.
pub fn encode_choice_idx(
    data: &mut AperCodecData,
    lb: i128,
    ub: i128,
    is_extensible: bool,
    idx: i128,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_choice_idx");
    check_bounds(idx, Some(lb), Some(ub), "Choice index")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended choice not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }
    encode_integer(data, Some(lb), Some(ub), false, idx, false)
}

/// Encode sequence header
pub fn encode_sequence_header(
    data: &mut AperCodecData,
    is_extensible: bool,
    optionals: &BitSlice<Msb0, u8>,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_sequence_header");

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended sequence not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }

    data.append_bits(optionals);
    Ok(())
}

/// Encode an Integer
pub fn encode_integer(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: i128,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_integer");
    check_bounds(value, lb, ub, "Integer")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended integer not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }

    match (lb, ub) {
        (None, _) => encode_unconstrained_whole_number(data, value),
        (Some(lb), None) => encode_semi_constrained_whole_number(data, lb, value),
        (Some(lb), Some(ub)) => encode_constrained_whole_number(data, lb, ub, value),
    }
}

/// Encode a BOOLEAN Value
///
/// Encodes a boolean value into the passed `AperCodecData` structure.
pub fn encode_bool(data: &mut AperCodecData, value: bool) -> Result<(), AperCodecError> {
    log::trace!("encode_bool");
    data.encode_bool(value);
    Ok(())
}

/// Encode an Enumerated Value
pub fn encode_enumerated(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: i128,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_enumerated");
    check_bounds(value as i128, lb, ub, "Enum value")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended enumerated not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }

    encode_integer(data, lb, ub, false, value, false)
}

/// Encode a Bit String
pub fn encode_bitstring(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    bit_string: &BitSlice<Msb0, u8>,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_bitstring");
    check_bounds(bit_string.len() as i128, lb, ub, "Bitstring length")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended bitstring not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }

    let length = bit_string.len();
    if length >= 16384 {
        return Err(AperCodecError::new(
            "Encode of fragmented bitstring not yet implemented",
        ));
    }

    encode_length_determinent(data, lb, ub, false, length)?;
    if length > 0 {
        if length > 16 {
            data.align();
        }
        data.append_bits(bit_string);
    }
    Ok(())
}

/// Encode an OCTET STRING
pub fn encode_octetstring(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    octet_string: &Vec<u8>,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_octetstring");
    check_bounds(octet_string.len() as i128, lb, ub, "Octet string length")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended octetstring not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }

    let length = octet_string.len();
    if length >= 16384 {
        return Err(AperCodecError::new(
            "Encode of fragmented octetstring not yet implemented",
        ));
    }

    encode_length_determinent(data, lb, ub, false, length)?;

    if length > 0 {
        if length > 2 {
            data.align();
        }
        data.append_bits(octet_string.view_bits());
    }
    Ok(())
}

// Encode a Length Determinent
pub fn encode_length_determinent(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    normally_small: bool,
    value: usize,
) -> Result<(), AperCodecError> {
    log::trace!("encode_length_determinent");
    check_bounds(value as i128, lb, ub, "Length determinent")?;

    if normally_small {
        return encode_normally_small_length_determinent(data, value);
    }

    match ub {
        Some(ub) if ub < 65_536 => {
            encode_constrained_whole_number(data, lb.unwrap_or(0), ub, value as i128)
        }
        _ => encode_indefinite_length_determinent(data, value),
    }
}

fn encode_string(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: &String,
    extended: bool,
) -> Result<(), AperCodecError> {
    check_bounds(value.len() as i128, lb, ub, "String length")?;

    if extended {
        return Err(AperCodecError::new(
            "Encode of extended visible string not yet implemented",
        ));
    }

    if is_extensible {
        data.encode_bool(extended);
    }
    encode_length_determinent(data, lb, ub, false, value.len())?;
    if value.len() > 2 {
        data.align();
    }
    data.append_bits(value.as_bits());
    Ok(())
}

/// Encode a VisibleString CharacterString Type.
pub fn encode_visible_string(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: &String,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_visible_string");
    encode_string(data, lb, ub, is_extensible, value, extended)
}

/// Encode a PrintableString CharacterString Type.
pub fn encode_printable_string(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: &String,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_printable_string");
    encode_string(data, lb, ub, is_extensible, value, extended)
}

/// Encode a UTF8String CharacterString Type.
pub fn encode_utf8_string(
    data: &mut AperCodecData,
    lb: Option<i128>,
    ub: Option<i128>,
    is_extensible: bool,
    value: &String,
    extended: bool,
) -> Result<(), AperCodecError> {
    log::trace!("encode_utf8_string");
    encode_string(data, lb, ub, is_extensible, value, extended)
}

fn check_bounds(
    value: i128,
    lb: Option<i128>,
    ub: Option<i128>,
    field: &str,
) -> Result<(), AperCodecError> {
    if let Some(l) = lb {
        if value < l {
            return Err(AperCodecError::new(format!(
                "{} {} is less than lower bound {}",
                field, value, l,
            )));
        }
    }

    if let Some(u) = ub {
        if value > u {
            return Err(AperCodecError::new(format!(
                "{} {} is greater than upper bound {}",
                field, value, u,
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_bool_always_success() {
        let mut data = AperCodecData::new();

        let result = encode_bool(&mut data, true);
        assert!(result.is_ok());
        assert_eq!(data.bits.len(), 1);
        assert_eq!(data.bits[0], true);
    }

    #[test]
    fn int_too_small() {
        assert!(encode_integer(&mut AperCodecData::new(), Some(1), None, false, 0, false).is_err());
    }

    #[test]
    fn int_too_big() {
        assert!(encode_integer(
            &mut AperCodecData::new(),
            Some(-1),
            Some(0),
            false,
            1,
            false
        )
        .is_err());
    }

    #[test]
    fn octetstring_too_small() {
        assert!(encode_octetstring(
            &mut AperCodecData::new(),
            Some(2),
            None,
            false,
            &vec![0],
            false
        )
        .is_err());
    }
    #[test]
    fn octetstring_too_big() {
        assert!(encode_octetstring(
            &mut AperCodecData::new(),
            None,
            Some(1),
            false,
            &vec![0, 0],
            false
        )
        .is_err());
    }

    #[test]
    fn string_too_small() {
        assert!(encode_visible_string(
            &mut AperCodecData::new(),
            Some(2),
            None,
            false,
            &"a".to_string(),
            false
        )
        .is_err());
    }

    #[test]
    fn string_too_big() {
        assert!(encode_visible_string(
            &mut AperCodecData::new(),
            None,
            Some(1),
            false,
            &"aa".to_string(),
            false
        )
        .is_err());
    }

    #[test]
    fn length_too_small() {
        assert!(
            encode_length_determinent(&mut AperCodecData::new(), Some(2), None, false, 1,).is_err()
        );
    }
    #[test]
    fn length_too_big() {
        assert!(
            encode_length_determinent(&mut AperCodecData::new(), None, Some(1), false, 2,).is_err()
        );
    }

    #[test]
    fn bitstring_too_small() {
        assert!(encode_bitstring(
            &mut AperCodecData::new(),
            Some(2),
            None,
            false,
            bits![Msb0,u8; 0],
            false
        )
        .is_err());
    }

    #[test]
    fn bitstring_too_big() {
        assert!(encode_bitstring(
            &mut AperCodecData::new(),
            None,
            Some(1),
            false,
            bits![Msb0,u8; 0, 0],
            false
        )
        .is_err());
    }
}
