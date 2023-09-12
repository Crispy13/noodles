use std::{error, fmt, mem};

use crate::record::{
    quality_scores::{self, Score},
    QualityScores,
};

/// An error when raw SAM record quality scores fail to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The length does not match the sequence length.
    LengthMismatch {
        /// The actual length.
        actual: usize,
        /// The expected length.
        expected: usize,
    },
    /// A score is invalid.
    InvalidScore(quality_scores::score::ParseError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidScore(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::LengthMismatch { actual, expected } => {
                write!(f, "length mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidScore(_) => write!(f, "invalid score"),
        }
    }
}

pub(super) fn parse_quality_scores(
    src: &[u8],
    sequence_len: usize,
    quality_scores: &mut QualityScores,
) -> Result<(), ParseError> {
    const OFFSET: u8 = b'!';

    if src.is_empty() {
        return Err(ParseError::Empty);
    } else if src.len() != sequence_len {
        return Err(ParseError::LengthMismatch {
            actual: src.len(),
            expected: sequence_len,
        });
    }

    let raw_quality_scores = Vec::from(mem::take(quality_scores));

    let mut raw_scores: Vec<_> = raw_quality_scores.into_iter().map(u8::from).collect();
    raw_scores.extend(src.iter().map(|n| n.wrapping_sub(OFFSET)));

    if let Some(n) = raw_scores.iter().copied().find(|&n| !is_valid_score(n)) {
        return Err(ParseError::InvalidScore(
            quality_scores::score::ParseError::Invalid(u32::from(n.wrapping_add(OFFSET))),
        ));
    }

    // SAFETY: Each score is guaranteed to be <= 93.
    let scores: Vec<_> = raw_scores.into_iter().map(Score).collect();
    *quality_scores = QualityScores::from(scores);

    Ok(())
}

fn is_valid_score(n: u8) -> bool {
    n <= Score::MAX.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quality_scores() -> Result<(), Box<dyn std::error::Error>> {
        let mut quality_scores = QualityScores::default();

        quality_scores.clear();
        parse_quality_scores(b"NDLS", 4, &mut quality_scores)?;
        let expected = QualityScores::try_from(vec![45, 35, 43, 50])?;
        assert_eq!(quality_scores, expected);

        quality_scores.clear();
        assert_eq!(
            parse_quality_scores(b"", 0, &mut quality_scores),
            Err(ParseError::Empty)
        );

        quality_scores.clear();
        assert_eq!(
            parse_quality_scores(b"NDLS", 2, &mut quality_scores),
            Err(ParseError::LengthMismatch {
                actual: 4,
                expected: 2
            })
        );

        quality_scores.clear();
        assert!(matches!(
            parse_quality_scores(&[0x07], 1, &mut quality_scores),
            Err(ParseError::InvalidScore(_))
        ));

        Ok(())
    }
}
