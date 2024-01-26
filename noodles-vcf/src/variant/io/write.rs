use std::io;

use crate::{Header, Record};

/// A variant format writer.
pub trait Write {
    /// Writes a VCF header.
    fn write_variant_header(&mut self, header: &Header) -> io::Result<()>;

    /// Writes a variant record.
    fn write_variant_record(&mut self, header: &Header, record: &Record) -> io::Result<()>;
}