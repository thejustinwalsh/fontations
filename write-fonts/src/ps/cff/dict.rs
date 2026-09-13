//! Serialization for PostScript DICTs.
//!
//! A DICT is a sequence of operands followed by the operator they belong to.
//! See "Table 3 Operand Encoding" and "Table 4 Operand Data" at
//! <https://adobe-type-tools.github.io/font-tech-notes/pdfs/5176.CFF.pdf>.
//!
//! Real operands are not written here. They are binary coded decimal, and
//! what a reader makes of one depends on the operator it belongs to:
//! [`read_fonts::ps::cff::dict`] follows FreeType, which reads `BlueScale`
//! and `FontMatrix` with a scaling of their own and keeps nine fractional
//! digits of anything else, so there is no one value a real operand is read
//! back as. A writer carrying real operands over from an existing DICT should
//! copy their bytes; one writing them from scratch has to choose a spelling,
//! the way fontTools does.

use read_fonts::ps::cff::dict::Operator;

/// Operand prefix for a 16 bit integer.
const SHORT_INT: u8 = 28;
/// Operand prefix for a 32 bit integer.
const LONG_INT: u8 = 29;

/// Writes an integer operand in the shortest available encoding.
pub fn write_int(value: i32, out: &mut Vec<u8>) {
    match value {
        -107..=107 => out.push((value + 139) as u8),
        108..=1131 => {
            let value = (value - 108) as u16;
            out.push(247 + (value >> 8) as u8);
            out.push(value as u8);
        }
        -1131..=-108 => {
            let value = (-value - 108) as u16;
            out.push(251 + (value >> 8) as u8);
            out.push(value as u8);
        }
        -32768..=32767 => write_short_int(value as i16, out),
        _ => write_long_int(value, out),
    }
}

/// Writes an integer operand in the three byte, 16 bit encoding.
///
/// Offsets are written with a fixed width encoding because their final value
/// depends on the size of the DICT that contains them.
pub fn write_short_int(value: i16, out: &mut Vec<u8>) {
    out.push(SHORT_INT);
    out.extend(value.to_be_bytes());
}

/// Writes an integer operand in the five byte, 32 bit encoding.
///
/// Offsets are written with a fixed width encoding because their final value
/// depends on the size of the DICT that contains them.
pub fn write_long_int(value: i32, out: &mut Vec<u8>) {
    out.push(LONG_INT);
    out.extend(value.to_be_bytes());
}

/// Writes an operator, ending the operands that precede it.
pub fn write_operator(operator: Operator, out: &mut Vec<u8>) {
    let (opcode, extended) = operator.opcode();
    out.push(opcode);
    out.extend(extended);
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_fonts::ps::cff::{dict::Token, stack::Number};

    fn round_trip(dict: &[u8]) -> Vec<Token> {
        read_fonts::ps::cff::dict::tokens(dict)
            .map(|token| token.unwrap())
            .collect()
    }

    fn round_trip_int(value: i32) -> Vec<u8> {
        let mut dict = Vec::new();
        write_int(value, &mut dict);
        assert_eq!(
            round_trip(&dict),
            [Token::Operand(Number::I32(value), None)]
        );
        dict
    }

    /// Integers use the encoding the spec gives for their magnitude.
    ///
    /// The boundaries are where the encoding changes width, so they are the
    /// values that catch an off by one in the ranges.
    #[test]
    fn write_int_picks_the_shortest_encoding() {
        assert_eq!(round_trip_int(0), [139]);
        assert_eq!(round_trip_int(107), [246]);
        assert_eq!(round_trip_int(-107), [32]);
        assert_eq!(round_trip_int(108), [247, 0]);
        assert_eq!(round_trip_int(1131), [250, 255]);
        assert_eq!(round_trip_int(-108), [251, 0]);
        assert_eq!(round_trip_int(-1131), [254, 255]);
        assert_eq!(round_trip_int(1132), [SHORT_INT, 0x04, 0x6c]);
        assert_eq!(round_trip_int(32767), [SHORT_INT, 0x7f, 0xff]);
        assert_eq!(round_trip_int(-32768), [SHORT_INT, 0x80, 0x00]);
        assert_eq!(round_trip_int(32768), [LONG_INT, 0, 0, 0x80, 0x00]);
        assert_eq!(round_trip_int(i32::MIN), [LONG_INT, 0x80, 0, 0, 0]);
        assert_eq!(round_trip_int(i32::MAX), [LONG_INT, 0x7f, 0xff, 0xff, 0xff]);
    }

    /// The fixed width encodings stay fixed width.
    #[test]
    fn write_int_fixed_widths() {
        let mut dict = Vec::new();
        write_short_int(0, &mut dict);
        write_long_int(0, &mut dict);
        assert_eq!(dict, [SHORT_INT, 0, 0, LONG_INT, 0, 0, 0, 0]);
        assert_eq!(
            round_trip(&dict),
            [
                Token::Operand(Number::I32(0), None),
                Token::Operand(Number::I32(0), None)
            ]
        );
    }

    /// Operators are written after the operands they consume.
    #[test]
    fn write_a_dict_entry() {
        let mut dict = Vec::new();
        write_int(391, &mut dict);
        write_operator(Operator::CharstringsOffset, &mut dict);
        write_int(0, &mut dict);
        write_operator(Operator::PaintType, &mut dict);
        assert_eq!(dict, [248, 27, 17, 139, 12, 5]);
        assert_eq!(
            round_trip(&dict),
            [
                Token::Operand(Number::I32(391), None),
                Token::Operator(Operator::CharstringsOffset),
                Token::Operand(Number::I32(0), None),
                Token::Operator(Operator::PaintType),
            ]
        );
    }
}
