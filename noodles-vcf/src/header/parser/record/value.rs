mod map;
mod string;

use std::{error, fmt};

use self::string::parse_string;
use crate::header::{
    record::{key, Key, Value},
    FileFormat, Number, Record,
};

/// An error returned when a VCF header record value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidFileFormat,
    InvalidInfo(map::info::ParseError),
    InvalidFilter(map::filter::ParseError),
    InvalidFormat(map::format::ParseError),
    InvalidAlternativeAllele(map::alternative_allele::ParseError),
    InvalidContig(map::contig::ParseError),
    InvalidOther(key::Other),
    FormatDefinitionMismatch {
        id: crate::record::genotypes::keys::Key,
        actual: (Number, crate::header::record::value::map::format::Type),
        expected: (Number, crate::header::record::value::map::format::Type),
    },
    InfoDefinitionMismatch {
        id: crate::record::info::field::Key,
        actual: (Number, crate::header::record::value::map::info::Type),
        expected: (Number, crate::header::record::value::map::info::Type),
    },
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidInfo(e) => Some(e),
            Self::InvalidFilter(e) => Some(e),
            Self::InvalidFormat(e) => Some(e),
            Self::InvalidAlternativeAllele(e) => Some(e),
            Self::InvalidContig(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFileFormat => write!(f, "invalid fileformat"),
            Self::InvalidInfo(e) => {
                write!(f, "invalid {}", key::INFO)?;

                if let Some(id) = e.id() {
                    write!(f, ": ID={id}")?;
                }

                Ok(())
            }
            Self::InvalidFilter(e) => {
                write!(f, "invalid {}", key::FILTER)?;

                if let Some(id) = e.id() {
                    write!(f, ": ID={id}")?;
                }

                Ok(())
            }
            Self::InvalidFormat(e) => {
                write!(f, "invalid {}", key::FORMAT)?;

                if let Some(id) = e.id() {
                    write!(f, ": ID={id}")?;
                }

                Ok(())
            }
            Self::InvalidAlternativeAllele(e) => {
                write!(
                    f,
                    "invalid alternative allele ({})",
                    key::ALTERNATIVE_ALLELE
                )?;

                if let Some(id) = e.id() {
                    write!(f, ": ID={id}")?;
                }

                Ok(())
            }
            Self::InvalidContig(e) => {
                write!(f, "invalid {}", key::CONTIG)?;

                if let Some(id) = e.id() {
                    write!(f, ": ID={id}")?;
                }

                Ok(())
            }
            Self::InvalidOther(key) => write!(f, "invalid other: {key}"),
            Self::FormatDefinitionMismatch {
                id,
                actual,
                expected,
            } => {
                let (actual_number, actual_type) = actual;
                let (expected_number, expected_type) = expected;

                write!(
                    f,
                    "{} definition mismatch for ID={id}: expected Number={},Type={}, got Number={},Type={}",
                    key::FORMAT,
                    expected_number, expected_type,
                    actual_number, actual_type,
                )
            }
            Self::InfoDefinitionMismatch {
                id,
                actual,
                expected,
            } => {
                let (actual_number, actual_type) = actual;
                let (expected_number, expected_type) = expected;

                write!(
                    f,
                    "{} definition mismatch for ID={id}: expected Number={},Type={}, got Number={},Type={}",
                    key::INFO,
                    expected_number, expected_type,
                    actual_number, actual_type,
                )
            }
        }
    }
}

pub(super) fn parse_value(
    src: &mut &[u8],
    file_format: FileFormat,
    key: Key,
) -> Result<Record, ParseError> {
    match key {
        key::FILE_FORMAT => parse_string(src)
            .map_err(|_| ParseError::InvalidFileFormat)
            .and_then(|s| s.parse().map_err(|_| ParseError::InvalidFileFormat))
            .map(Record::FileFormat),
        key::INFO => {
            let (id, map) = map::parse_info(src, file_format).map_err(ParseError::InvalidInfo)?;
            validate_info_definition(file_format, &id, map.number(), map.ty())?;
            Ok(Record::Info(id, map))
        }
        key::FILTER => map::parse_filter(src)
            .map(|(id, map)| Record::Filter(id, map))
            .map_err(ParseError::InvalidFilter),
        key::FORMAT => {
            let (id, map) =
                map::parse_format(src, file_format).map_err(ParseError::InvalidFormat)?;
            validate_format_definition(file_format, &id, map.number(), map.ty())?;
            Ok(Record::Format(id, map))
        }
        key::ALTERNATIVE_ALLELE => map::parse_alternative_allele(src)
            .map(|(id, map)| Record::AlternativeAllele(id, map))
            .map_err(ParseError::InvalidAlternativeAllele),
        key::CONTIG => map::parse_contig(src)
            .map(|(id, map)| Record::Contig(id, map))
            .map_err(ParseError::InvalidContig),
        Key::Other(k) => {
            let v = if map::is_map(src) {
                map::parse_other(src)
                    .map(Value::from)
                    .map_err(|_| ParseError::InvalidOther(k.clone()))?
            } else {
                parse_string(src)
                    .map(String::from)
                    .map(Value::from)
                    .map_err(|_| ParseError::InvalidOther(k.clone()))?
            };

            Ok(Record::Other(k, v))
        }
    }
}

fn validate_format_definition(
    file_format: FileFormat,
    id: &crate::record::genotypes::keys::Key,
    actual_number: Number,
    actual_type: crate::header::record::value::map::format::Type,
) -> Result<(), ParseError> {
    use crate::header::record::value::map::format::definition::definition;

    if let Some((expected_number, expected_type, _)) = definition(file_format, id) {
        if actual_number != expected_number || actual_type != expected_type {
            return Err(ParseError::FormatDefinitionMismatch {
                id: id.clone(),
                actual: (actual_number, actual_type),
                expected: (expected_number, expected_type),
            });
        }
    }

    Ok(())
}

fn validate_info_definition(
    file_format: FileFormat,
    id: &crate::record::info::field::Key,
    actual_number: Number,
    actual_type: crate::header::record::value::map::info::Type,
) -> Result<(), ParseError> {
    use crate::header::record::value::map::info::definition::definition;

    if let Some((expected_number, expected_type, _)) = definition(file_format, id) {
        if actual_number != expected_number || actual_type != expected_type {
            return Err(ParseError::InfoDefinitionMismatch {
                id: id.clone(),
                actual: (actual_number, actual_type),
                expected: (expected_number, expected_type),
            });
        }
    }

    Ok(())
}
