use super::{VarInt, VarIntError};

use std::io::{Error as IoError, Read, Result as IoResult};

pub enum ElementDataSize {
    Known(VarInt),
    Unknown,
}

impl ElementDataSize {
    pub fn from_reader<R: Read>(mut src: R, max_octets: Option<usize>) -> Result<Self, VarIntError> {
        let (var_int, bytes_taken) = VarInt::from_reader_with_octets(src, max_octets)?;

        Ok(if var_int.is_full(bytes_taken) {
            ElementDataSize::Unknown
        } else {
            ElementDataSize::Known(var_int)
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unknown_size() {
        let mut test_1 = [0b1111_1111];
        let mut test_2 = [0b0111_1111, 0b1111_1111];
        let mut test_3 = [0b0011_1111, 0b1111_1111, 0b1111_1111];
        let mut test_4 = [0b0001_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111];

        assert!(matches!(
            ElementDataSize::from_reader(&test_1[..], None),
            Ok(ElementDataSize::Unknown)
        ));

        assert!(matches!(
            ElementDataSize::from_reader(&test_2[..], None),
            Ok(ElementDataSize::Unknown)
        ));

        assert!(matches!(
            ElementDataSize::from_reader(&test_3[..], None),
            Ok(ElementDataSize::Unknown)
        ));

        assert!(matches!(
            ElementDataSize::from_reader(&test_4[..], None),
            Ok(ElementDataSize::Unknown)
        ));
    }
}