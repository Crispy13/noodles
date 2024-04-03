//! VCF record info field value.

mod array;

pub use self::array::Array;

use std::{error, fmt, num, str};

use super::MISSING_VALUE;
use crate::{
    header::{
        record::value::{
            map::{info::Type, Info},
            Map,
        },
        Number,
    },
    record::value::percent_decode,
};

const DELIMITER: char = ',';

/// A VCF record info field value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// An 32-bit integer.
    Integer(i32),
    /// A single-precision floating-point.
    Float(f32),
    /// A boolean.
    Flag,
    /// A character.
    Character(char),
    /// A string.
    String(String),
    /// An array.
    Array(Array),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::Flag => Ok(()),
            Self::Character(c) => write!(f, "{c}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Array(array) => write!(f, "{array}"),
        }
    }
}

/// An error returned when a raw VCF record info field value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The field cardinality is invalid for the type.
    InvalidNumberForType(Number, Type),
    /// The integer is invalid.
    InvalidInteger(num::ParseIntError),
    /// The floating-point is invalid.
    InvalidFloat(num::ParseFloatError),
    /// The flag is invalid.
    InvalidFlag,
    /// The character is invalid.
    InvalidCharacter,
    /// The string is invalid.
    InvalidString(str::Utf8Error),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidInteger(e) => Some(e),
            Self::InvalidFloat(e) => Some(e),
            Self::InvalidString(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumberForType(number, ty) => {
                write!(f, "invalid number {number:?} for type {ty:?}")
            }
            Self::InvalidInteger(_) => f.write_str("invalid integer"),
            Self::InvalidFloat(_) => f.write_str("invalid float"),
            Self::InvalidFlag => f.write_str("invalid flag"),
            Self::InvalidCharacter => f.write_str("invalid character"),
            Self::InvalidString(_) => f.write_str("invalid string"),
        }
    }
}

impl Value {
    /// Parses a raw info field value with the given info header record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     header::record::value::{map::Info, Map},
    ///     record::info::field::{key, Value},
    /// };
    ///
    /// let info = Map::<Info>::from(key::SAMPLES_WITH_DATA_COUNT);
    /// assert_eq!(Value::from_str_info("1", &info), Ok(Value::Integer(1)));
    /// ```
    pub fn from_str_info(s: &str, info: &Map<Info>) -> Result<Self, ParseError> {
        parse(info.number(), info.ty(), s)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Self::Integer(n)
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Self::Float(n)
    }
}

impl From<char> for Value {
    fn from(c: char) -> Self {
        Self::Character(c)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<Vec<Option<i32>>> for Value {
    fn from(values: Vec<Option<i32>>) -> Self {
        Self::Array(Array::Integer(values))
    }
}

impl From<Vec<Option<f32>>> for Value {
    fn from(values: Vec<Option<f32>>) -> Self {
        Self::Array(Array::Float(values))
    }
}

impl From<Vec<Option<char>>> for Value {
    fn from(values: Vec<Option<char>>) -> Self {
        Self::Array(Array::Character(values))
    }
}

impl From<Vec<Option<String>>> for Value {
    fn from(values: Vec<Option<String>>) -> Self {
        Self::Array(Array::String(values))
    }
}

impl TryFrom<(Number, Type, &str)> for Value {
    type Error = ParseError;

    fn try_from((number, ty, s): (Number, Type, &str)) -> Result<Self, Self::Error> {
        parse(number, ty, s)
    }
}

fn parse(number: Number, ty: Type, s: &str) -> Result<Value, ParseError> {
    match ty {
        Type::Integer => match number {
            Number::Count(0) => Err(ParseError::InvalidNumberForType(number, ty)),
            Number::Count(1) => parse_i32(s),
            _ => parse_i32_array(s),
        },
        Type::Float => match number {
            Number::Count(0) => Err(ParseError::InvalidNumberForType(number, ty)),
            Number::Count(1) => parse_f32(s),
            _ => parse_f32_array(s),
        },
        Type::Flag => match number {
            Number::Count(0) => parse_flag(s),
            _ => Err(ParseError::InvalidNumberForType(number, ty)),
        },
        Type::Character => match number {
            Number::Count(0) => Err(ParseError::InvalidNumberForType(number, ty)),
            Number::Count(1) => parse_char(s),
            _ => parse_char_array(s),
        },
        Type::String => match number {
            Number::Count(0) => Err(ParseError::InvalidNumberForType(number, ty)),
            Number::Count(1) => parse_string(s),
            _ => parse_string_array(s),
        },
    }
}

fn parse_i32(s: &str) -> Result<Value, ParseError> {
    s.parse()
        .map(Value::Integer)
        .map_err(ParseError::InvalidInteger)
}

fn parse_i32_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => t.parse().map(Some).map_err(ParseError::InvalidInteger),
        })
        .collect::<Result<_, _>>()
        .map(|values| Value::Array(Array::Integer(values)))
}

fn parse_f32(s: &str) -> Result<Value, ParseError> {
    s.parse()
        .map(Value::Float)
        .map_err(ParseError::InvalidFloat)
}

fn parse_f32_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => t.parse().map(Some).map_err(ParseError::InvalidFloat),
        })
        .collect::<Result<_, _>>()
        .map(|values| Value::Array(Array::Float(values)))
}

fn parse_flag(s: &str) -> Result<Value, ParseError> {
    if s.is_empty() {
        Ok(Value::Flag)
    } else {
        Err(ParseError::InvalidFlag)
    }
}

fn parse_raw_char(s: &str) -> Result<char, ParseError> {
    let mut chars = s.chars();

    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            return Ok(c);
        }
    }

    Err(ParseError::InvalidCharacter)
}

fn parse_char(s: &str) -> Result<Value, ParseError> {
    parse_raw_char(s).map(Value::Character)
}

fn parse_char_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => parse_raw_char(t).map(Some),
        })
        .collect::<Result<_, _>>()
        .map(|values| Value::Array(Array::Character(values)))
}

fn parse_string(s: &str) -> Result<Value, ParseError> {
    percent_decode(s)
        .map(|t| Value::String(t.into()))
        .map_err(ParseError::InvalidString)
}

fn parse_string_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => percent_decode(t)
                .map(|u| Some(u.into()))
                .map_err(ParseError::InvalidString),
        })
        .collect::<Result<_, _>>()
        .map(|values| Value::Array(Array::String(values)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let value = Value::from(2);
        assert_eq!(value.to_string(), "2");

        let value = Value::from(0.333);
        assert_eq!(value.to_string(), "0.333");

        assert_eq!(Value::Flag.to_string(), "");

        let value = Value::from('n');
        assert_eq!(value.to_string(), "n");

        let value = Value::from("noodles");
        assert_eq!(value.to_string(), "noodles");

        let value = Value::from(vec![Some(2)]);
        assert_eq!(value.to_string(), "2");

        let value = Value::from(vec![Some(2), Some(5)]);
        assert_eq!(value.to_string(), "2,5");

        let value = Value::from(vec![Some(2), None]);
        assert_eq!(value.to_string(), "2,.");

        let value = Value::from(vec![Some(0.333)]);
        assert_eq!(value.to_string(), "0.333");

        let value = Value::from(vec![Some(0.333), Some(0.667)]);
        assert_eq!(value.to_string(), "0.333,0.667");

        let value = Value::from(vec![Some(0.333), None]);
        assert_eq!(value.to_string(), "0.333,.");

        let value = Value::from(vec![Some('n')]);
        assert_eq!(value.to_string(), "n");

        let value = Value::from(vec![Some('n'), Some('d'), Some('l'), Some('s')]);
        assert_eq!(value.to_string(), "n,d,l,s");

        let value = Value::from(vec![Some('n'), Some('d'), Some('l'), None]);
        assert_eq!(value.to_string(), "n,d,l,.");

        let value = Value::from(vec![Some(String::from("noodles"))]);
        assert_eq!(value.to_string(), "noodles");

        let value = Value::from(vec![
            Some(String::from("noodles")),
            Some(String::from("vcf")),
        ]);
        assert_eq!(value.to_string(), "noodles,vcf");

        let value = Value::from(vec![Some(String::from("noodles")), None]);
        assert_eq!(value.to_string(), "noodles,.");
    }

    #[test]
    fn test_parse_with_integer() {
        assert_eq!(
            parse(Number::Count(0), Type::Integer, "8"),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Integer
            ))
        );

        assert_eq!(
            parse(Number::Count(1), Type::Integer, "8"),
            Ok(Value::from(8))
        );

        assert_eq!(
            parse(Number::Count(2), Type::Integer, "8,13"),
            Ok(Value::from(vec![Some(8), Some(13)]))
        );
        assert_eq!(
            parse(Number::Count(2), Type::Integer, "8,."),
            Ok(Value::from(vec![Some(8), None]))
        );
    }

    #[test]
    fn test_parse_with_float() {
        assert_eq!(
            parse(Number::Count(0), Type::Float, "0.333"),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Float
            ))
        );

        assert_eq!(
            parse(Number::Count(1), Type::Float, "0.333"),
            Ok(Value::from(0.333))
        );

        assert_eq!(
            parse(Number::Count(2), Type::Float, "0.333,0.667"),
            Ok(Value::from(vec![Some(0.333), Some(0.667)]))
        );
        assert_eq!(
            parse(Number::Count(2), Type::Float, "0.333,."),
            Ok(Value::from(vec![Some(0.333), None]))
        );
    }

    #[test]
    fn test_parse_with_flag() {
        assert_eq!(parse(Number::Count(0), Type::Flag, ""), Ok(Value::Flag));

        assert_eq!(
            parse(Number::Count(0), Type::Flag, "true"),
            Err(ParseError::InvalidFlag)
        );

        assert_eq!(
            parse(Number::Count(1), Type::Flag, ""),
            Err(ParseError::InvalidNumberForType(
                Number::Count(1),
                Type::Flag
            ))
        );
    }

    #[test]
    fn test_parse_with_character() {
        assert_eq!(
            parse(Number::Count(0), Type::Character, "n"),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Character
            ))
        );

        assert_eq!(
            parse(Number::Count(1), Type::Character, "n"),
            Ok(Value::from('n'))
        );

        assert_eq!(
            parse(Number::Count(2), Type::Character, "n,d,l,s"),
            Ok(Value::from(vec![
                Some('n'),
                Some('d'),
                Some('l'),
                Some('s')
            ]))
        );
        assert_eq!(
            parse(Number::Count(2), Type::Character, "n,d,l,."),
            Ok(Value::from(vec![Some('n'), Some('d'), Some('l'), None]))
        );
    }

    #[test]
    fn test_parse_with_string() {
        assert_eq!(
            parse(Number::Count(0), Type::String, "noodles"),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::String
            ))
        );

        assert_eq!(
            parse(Number::Count(1), Type::String, "noodles"),
            Ok(Value::from("noodles"))
        );
        assert_eq!(
            parse(Number::Count(1), Type::String, "8%25"),
            Ok(Value::from("8%"))
        );

        assert_eq!(
            parse(Number::Count(2), Type::String, "noodles,vcf"),
            Ok(Value::from(vec![
                Some(String::from("noodles")),
                Some(String::from("vcf"))
            ]))
        );
        assert_eq!(
            parse(Number::Count(2), Type::String, "noodles,."),
            Ok(Value::from(vec![Some(String::from("noodles")), None]))
        );
        assert_eq!(
            parse(Number::Count(2), Type::String, "8%25,13%25"),
            Ok(Value::from(vec![
                Some(String::from("8%")),
                Some(String::from("13%"))
            ]))
        );
    }
}
