use std::{
    cmp::min,
    ops::{AddAssign, Shl, ShrAssign},
};

pub use bitfield_impl::{bitfield, generate_specifiers};

// We are actually storing the value in big endian to avoid the reverse
// We could actually use https://doc.rust-lang.org/std/primitive.u8.html#method.to_be

pub trait Specifier {
    const BITS: usize;
    type TypeRepr: From<u8>
        + AddAssign
        + Shl<usize, Output = Self::TypeRepr>
        + ShrAssign<usize>
        + LastByte;

    fn get(data: &[u8], offset: usize) -> Self::TypeRepr {
        let mut byte_idx = offset / 8;
        let mut start_offset = offset % 8;
        let mut rem_bits = Self::BITS;
        let mut result = Self::TypeRepr::from(0u8);
        while rem_bits > 0 {
            let rem_bits_current_byte = min(8 - start_offset, rem_bits);
            let value: u8 = if rem_bits_current_byte == 8 {
                data[byte_idx]
            } else {
                data[byte_idx].value_from_bits(start_offset, rem_bits_current_byte)
            };
            result += Self::TypeRepr::from(value) << (Self::BITS - rem_bits);
            rem_bits -= rem_bits_current_byte;
            byte_idx += 1;
            start_offset = 0;
        }
        result
    }

    fn set(data: &mut [u8], offset: usize, mut value: Self::TypeRepr) {
        let mut byte_idx = offset / 8;
        let mut start_offset = offset % 8;
        let mut rem_bits = Self::BITS;
        while rem_bits > 0 {
            let rem_bits_current_byte = min(8 - start_offset, rem_bits);
            let new_byte: u8 = if rem_bits_current_byte == 8 {
                value.last_byte()
            } else {
                let start = data[byte_idx].value_from_bits(0, start_offset);
                let middle = value.last_byte() << start_offset;
                let end_offset = start_offset + rem_bits_current_byte;
                let end = if end_offset == 8 {
                    // Prevent shl overflow
                    0
                } else {
                    data[byte_idx].value_from_bits(end_offset, 8 - end_offset) << end_offset
                };
                end + middle + start
            };
            data[byte_idx] = new_byte;
            value >>= rem_bits_current_byte;
            rem_bits -= rem_bits_current_byte;
            byte_idx += 1;
            start_offset = 0;
        }
    }
}

pub trait BitsExt: Copy + Sized {
    fn value_from_bits(self, start: usize, len: usize) -> Self;
}

impl BitsExt for u8 {
    fn value_from_bits(self, start: usize, len: usize) -> Self {
        match (start, len) {
            (_, 0) | (8, _) => 0,
            _ => {
                // We need to prevent `shl` to overflow
                let value = if start + len >= u8::BITS as usize {
                    self
                } else {
                    let mask = (1 << len) - 1;
                    self & (mask << start)
                };
                value >> start
            }
        }
    }
}

// TryFrom won't work, you need this ad-hoc typeclass :(
pub trait LastByte: Copy {
    fn last_byte(self) -> u8;
}

bitfield_impl::generate_specifiers!();

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0b0000_0000, 0, 0 => 0)]
    #[test_case(0b0000_0000, 1, 0 => 0)]
    #[test_case(0b0111_1111, 0, 7 => 127)]
    #[test_case(0b0001_1001, 0, 4 => 9)]
    #[test_case(0b0001_1001, 2, 4 => 6)]
    #[test_case(0b1111_1111, 0, 8 => 255)]
    #[test_case(0b1111_1111, 2, 6 => 63)]
    #[test_case(0b1111_1111, 8, 0 => 0)]
    #[test_case(0b1111_1111, 2, 0 => 0)]
    #[test_case(0b1111_1111, 7, 1 => 1)]
    fn bits_test(byte: u8, start: usize, len: usize) -> u8 {
        byte.value_from_bits(start, len)
    }

    #[test]
    fn specifier_test() {
        let mut data: [u8; 1] = [0b0000_0010];
        assert_eq!(B1::get(&data, 0), 0);
        assert_eq!(B1::get(&data, 1), 1);
        assert_eq!(B1::get(&data, 2), 0);
        B1::set(&mut data, 0, 1);
        assert_eq!(B1::get(&data, 0), 1);

        let mut data: [u8; 1] = [0b0100_1010];
        assert_eq!(B4::get(&data, 0), 0b1010);
        assert_eq!(B4::get(&data, 2), 0b0010);
        assert_eq!(B4::get(&data, 3), 0b1001);
        assert_eq!(B4::get(&data, 4), 0b0100);
        B4::set(&mut data, 0, 0b0000);
        B4::set(&mut data, 1, 0b1111);
        assert_eq!(B4::get(&data, 0), 0b1110);
    }
}
