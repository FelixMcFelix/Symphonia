use super::{VarInt, VarIntError};

use std::io::{Error as IoError, Read, Result as IoResult};

pub enum ElementIdError {
    Parse(VarIntError),
    AllZero,
    AllOnes,
    ReprTooBig(ElementId),
}

impl From<VarIntError> for ElementIdError {
    fn from(e: VarIntError) -> Self {
        Self::Parse(e)
    }
}

pub struct ElementId(VarInt);

impl ElementId {
    pub fn from_reader<R: Read>(mut src: R, max_octets: Option<usize>) -> Result<Self, ElementIdError> {
        let (var_int, bytes_taken) = VarInt::from_reader_with_octets(src, max_octets)?;

        if var_int.is_zero() {
            Err(ElementIdError::AllZero)
        } else if var_int.is_full(bytes_taken) {
            Err(ElementIdError::AllOnes)
        } else if var_int.octets_needed() < bytes_taken && !var_int.is_full(bytes_taken-1) {
            Err(ElementIdError::ReprTooBig(Self(var_int)))
        } else {
            Ok(Self(var_int))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn element_id_rejects_all_ones() {
        let mut test_1 = [0b1111_1111];

        assert!(matches!(
            ElementId::from_reader(&test_1[..], None),
            Err(ElementIdError::AllOnes)
        ));
    }

    #[test]
    fn element_id_rejects_all_zeros() {
        let mut test_1 = [0b1000_0000];

        assert!(matches!(
            ElementId::from_reader(&test_1[..], None),
            Err(ElementIdError::AllZero)
        ));
    }

    #[test]
    fn element_id_all_ones_allows_more_octets() {
        let mut test_1 = [0b0100_0000, 0b0111_1111];

        assert!(matches!(
            ElementId::from_reader(&test_1[..], None),
            Ok(ElementId(VarInt::Native(0b0111_1111)))
        ));
    }

    #[test]
    fn element_id_reject_excess_bytes() {
        let mut test_1 = [0b0100_0000, 0b0111_1110];

        assert!(matches!(
            ElementId::from_reader(&test_1[..], None),
            Err(ElementIdError::ReprTooBig(ElementId(VarInt::Native(0b0111_1110))))
        ));
    }
}