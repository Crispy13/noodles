//! Raw BAM record data.

pub mod field;

use std::{borrow::Borrow, io, iter};

use noodles_sam::{
    self as sam,
    alignment::record::data::field::{Tag, Value},
};

use self::field::decode_field;

/// Raw BAM record data.
pub struct Data<'a>(&'a [u8]);

impl<'a> Data<'a> {
    pub(super) fn new(src: &'a [u8]) -> Self {
        Self(src)
    }

    /// Returns whether there are any fields.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the value of the given tag.
    pub fn get<K>(&self, tag: &K) -> Option<io::Result<Value<'_>>>
    where
        K: Borrow<[u8; 2]>,
    {
        for result in self.iter() {
            match result {
                Ok((t, value)) => {
                    if &t == tag.borrow() {
                        return Some(Ok(value));
                    }
                }
                Err(e) => return Some(Err(e)),
            };
        }

        None
    }

    /// Returns an iterator over all tag-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = io::Result<(Tag, Value<'_>)>> + '_ {
        let mut src = self.0;

        iter::from_fn(move || {
            if src.is_empty() {
                None
            } else {
                Some(decode_field(&mut src))
            }
        })
    }
}

impl<'a> sam::alignment::record::Data for Data<'a> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn get(&self, tag: &Tag) -> Option<io::Result<Value<'_>>> {
        self.get(tag)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<(Tag, Value<'_>)>> + '_> {
        Box::new(self.iter())
    }
}

impl<'a> AsRef<[u8]> for Data<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> TryFrom<Data<'a>> for sam::alignment::record_buf::Data {
    type Error = io::Error;

    fn try_from(bam_data: Data<'a>) -> Result<Self, Self::Error> {
        use crate::record::codec::decoder::get_data;

        let mut src = bam_data.0;
        let mut sam_data = Self::default();
        get_data(&mut src, &mut sam_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(sam_data)
    }
}

pub(super) fn get_raw_cigar<'a>(src: &mut &'a [u8]) -> io::Result<Option<&'a [u8]>> {
    use noodles_sam::alignment::record::data::field::{tag, Type};

    use self::field::{
        decode_tag, decode_type, decode_value,
        value::array::{decode_raw_array, decode_subtype},
    };

    fn get_array_field<'a>(src: &mut &'a [u8]) -> io::Result<Option<(Tag, &'a [u8])>> {
        let tag = decode_tag(src)?;
        let ty = decode_type(src)?;

        if ty == Type::Array {
            let subtype = decode_subtype(src)?;
            let buf = decode_raw_array(src, subtype)?;
            Ok(Some((tag, buf)))
        } else {
            decode_value(src, ty)?;
            Ok(None)
        }
    }

    while !src.is_empty() {
        if let Some((tag::CIGAR, buf)) = get_array_field(src)? {
            return Ok(Some(buf));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use sam::alignment::record::data::field::tag;

    use super::*;

    #[test]
    fn test_get() -> io::Result<()> {
        let data = Data::new(&[b'N', b'H', b'C', 0x01]);

        assert!(data.get(&tag::ALIGNMENT_HIT_COUNT).is_some());
        assert!(data.get(&[b'N', b'H']).is_some());

        assert!(data.get(&tag::COMMENT).is_none());

        Ok(())
    }

    #[test]
    fn test_iter() -> io::Result<()> {
        let data = Data::new(&[]);
        assert!(data.iter().next().is_none());

        let data = Data::new(&[b'N', b'H', b'C', 0x01]);
        let actual: Vec<_> = data.iter().collect::<io::Result<_>>()?;

        assert_eq!(actual.len(), 1);

        let (actual_tag, actual_value) = &actual[0];
        assert_eq!(actual_tag, &tag::ALIGNMENT_HIT_COUNT);
        assert!(matches!(actual_value, Value::UInt8(1)));

        Ok(())
    }
}
