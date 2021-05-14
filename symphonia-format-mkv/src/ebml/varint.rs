use num_bigint::BigUint;
use std::io::{Error as IoError, Read, Result as IoResult};

// VINT: zeros + one + BE-(7x that) unsigned number
// said number is internally zero-padded.
// arbitrary content


#[derive(Debug)]
pub enum VarIntError {
    Io(IoError),
    ExceededSize,
}

impl From<IoError> for VarIntError {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VarInt {
    Native(u64),
    Big(BigUint),
}

impl VarInt {
    pub fn from_reader<R: Read>(mut src: R, max_octets: Option<usize>) -> Result<Self, VarIntError> {
        Self::from_reader_with_octets(src, max_octets)
            .map(|o| o.0)
    }

    pub fn from_reader_with_octets<R: Read>(mut src: R, max_octets: Option<usize>) -> Result<(Self, usize), VarIntError> {
        let mut byte_space = [0u8; 8];
        let mut i = 0;

        let zero_count = loop {
            src.read_exact(&mut byte_space[..1])?;
            let lzs = byte_space[0].leading_zeros();

            if lzs < 8 {
                break (8 * i) + lzs;
            }

            i += 1;
        };

        if let Some(max_octets) = max_octets {
            if zero_count as usize + 1 > max_octets {
                return Err(VarIntError::ExceededSize)
            }
        }

        if zero_count < 8 {
            src.read_exact(&mut byte_space[1..(1+zero_count as usize)])?;

            let cancel_bit: u64 = 1 << 63 - zero_count;
            let shr_amt = 8 * (8 - (zero_count + 1));

            let out = VarInt::Native((u64::from_be_bytes(byte_space) & !cancel_bit) >> shr_amt);

            Ok((out, 1+zero_count as usize))
        } else {
            //bigint time

            unimplemented!()
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Native(n) => *n == 0,
            Self::Big(b) => unimplemented!(),
        }
    }

    pub fn is_full(&self, octet_count: usize) -> bool {
        match self {
            Self::Native(n) => *n as usize == Self::max_val_for_octet_count(octet_count),
            Self::Big(b) => unimplemented!(),
        }
    }

    pub fn max_val_for_octet_count(octets: usize) -> usize {
        (1 << (octets * 7)) - 1
    }

    pub fn octets_needed_for_val(val: u64) -> usize {
        let bits_to_repr_value = std::mem::size_of::<u64>() * 8 - val.leading_zeros() as usize;
        (bits_to_repr_value + 6) / 7
    }

    pub fn octets_needed(&self) -> usize {
        match self {
            Self::Native(n) => Self::octets_needed_for_val(*n),
            _ => unimplemented!(),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_octet_var_ints() {
        let test_1: [u8; 1] = [0b1000_0000]; // 0
        let test_2: [u8; 1] = [0b1000_0010]; // 2

        assert_eq!(
            VarInt::from_reader(&test_1[..], None).unwrap(),
            VarInt::Native(0)
        );

        assert!(matches!(
            VarInt::from_reader(&test_2[..], None),
            Ok(VarInt::Native(2))
        ));
    }

    #[test]
    fn dual_octet_var_ints() {
        let test_1: [u8; 2] = [0b0100_0000, 0b0000_0010]; // 2

        assert!(matches!(
            VarInt::from_reader(&test_1[..], None),
            Ok(VarInt::Native(2))
        ));
    }

    #[test]
    fn tri_octet_var_ints() {
        let test_1: [u8; 3] = [0b0010_0000, 0b0000_0000, 0b0000_0010]; // 2

        assert!(matches!(
            VarInt::from_reader(&test_1[..], None),
            Ok(VarInt::Native(2))
        ));
    }

    #[test]
    fn quad_octet_var_ints() {
        let test_1: [u8; 4] = [0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0010]; // 2

        assert!(matches!(
            VarInt::from_reader(&test_1[..], None),
            Ok(VarInt::Native(2))
        ));
    }

    #[test]
    fn max_size_limit_holds() {
        let mut test_1 = [0u8; 20];
        test_1[2] = 0b0001_0000;

        assert!(matches!(
            VarInt::from_reader(&test_1[..], Some(4)),
            Err(VarIntError::ExceededSize)
        ));
    }
}