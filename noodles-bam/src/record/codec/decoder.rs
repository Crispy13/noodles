//! BAM record decoder.

pub(crate) mod cigar;
pub mod data;
mod flags;
mod mapping_quality;
mod position;
mod quality_scores;
pub(crate) mod read_name;
mod reference_sequence_id;
pub(crate) mod sequence;
mod template_length;

pub(crate) use self::{
    cigar::get_cigar, data::get_data, flags::get_flags, mapping_quality::get_mapping_quality,
    position::get_position, quality_scores::get_quality_scores, read_name::get_read_name,
    reference_sequence_id::get_reference_sequence_id, sequence::get_sequence,
};

use std::{error, fmt, mem};

use bytes::Buf;
use noodles_sam::{self as sam, alignment::Record};

use self::template_length::get_template_length;

/// An error when a raw BAM record fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The reference sequence ID is invalid.
    InvalidReferenceSequenceId(reference_sequence_id::ParseError),
    /// The position is invalid.
    InvalidPosition(position::ParseError),
    /// The mapping quality is invalid.
    InvalidMappingQuality(mapping_quality::ParseError),
    /// The flags are invalid.
    InvalidFlags(flags::ParseError),
    /// The mate reference sequence ID is invalid.
    InvalidMateReferenceSequenceId(reference_sequence_id::ParseError),
    /// The mate position is invalid.
    InvalidMatePosition(position::ParseError),
    /// The template length is invalid.
    InvalidTemplateLength(template_length::ParseError),
    /// The read name is invalid.
    InvalidReadName(read_name::ParseError),
    /// The CIGAR is invalid.
    InvalidCigar(cigar::ParseError),
    /// The sequence is invalid.
    InvalidSequence(sequence::ParseError),
    /// The quality scores are invalid.
    InvalidQualityScores(quality_scores::ParseError),
    /// The data is invalid.
    InvalidData(data::ParseError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidReferenceSequenceId(e) => Some(e),
            Self::InvalidPosition(e) => Some(e),
            Self::InvalidMappingQuality(e) => Some(e),
            Self::InvalidFlags(e) => Some(e),
            Self::InvalidMateReferenceSequenceId(e) => Some(e),
            Self::InvalidMatePosition(e) => Some(e),
            Self::InvalidTemplateLength(e) => Some(e),
            Self::InvalidReadName(e) => Some(e),
            Self::InvalidCigar(e) => Some(e),
            Self::InvalidSequence(e) => Some(e),
            Self::InvalidQualityScores(e) => Some(e),
            Self::InvalidData(e) => Some(e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReferenceSequenceId(_) => write!(f, "invalid reference sequence ID"),
            Self::InvalidPosition(_) => write!(f, "invalid position"),
            Self::InvalidMappingQuality(_) => write!(f, "invalid mapping quality"),
            Self::InvalidFlags(_) => write!(f, "invalid flags"),
            Self::InvalidMateReferenceSequenceId(_) => {
                write!(f, "invalid mate reference sequence ID")
            }
            Self::InvalidMatePosition(_) => write!(f, "invalid mate position"),
            Self::InvalidTemplateLength(_) => write!(f, "invalid template length"),
            Self::InvalidReadName(_) => write!(f, "invalid read name"),
            Self::InvalidCigar(_) => write!(f, "invalid CIGAR"),
            Self::InvalidSequence(_) => write!(f, "invalid sequence"),
            Self::InvalidQualityScores(_) => write!(f, "invalid quality scores"),
            Self::InvalidData(_) => write!(f, "invalid data"),
        }
    }
}

pub(crate) fn decode_record<B>(
    src: &mut B,
    header: &sam::Header,
    record: &mut Record,
) -> Result<(), ParseError>
where
    B: Buf,
{
    let n_ref = header.reference_sequences().len();

    *record.reference_sequence_id_mut() =
        get_reference_sequence_id(src, n_ref).map_err(ParseError::InvalidReferenceSequenceId)?;

    *record.alignment_start_mut() = get_position(src).map_err(ParseError::InvalidPosition)?;

    let l_read_name = read_name::get_length(src).map_err(ParseError::InvalidReadName)?;

    *record.mapping_quality_mut() =
        get_mapping_quality(src).map_err(ParseError::InvalidMappingQuality)?;

    // Discard bin.
    src.advance(mem::size_of::<u16>());

    let n_cigar_op = cigar::get_op_count(src).map_err(ParseError::InvalidCigar)?;

    *record.flags_mut() = get_flags(src).map_err(ParseError::InvalidFlags)?;

    let l_seq = sequence::get_length(src).map_err(ParseError::InvalidSequence)?;

    *record.mate_reference_sequence_id_mut() = get_reference_sequence_id(src, n_ref)
        .map_err(ParseError::InvalidMateReferenceSequenceId)?;

    *record.mate_alignment_start_mut() =
        get_position(src).map_err(ParseError::InvalidMatePosition)?;

    *record.template_length_mut() =
        get_template_length(src).map_err(ParseError::InvalidTemplateLength)?;

    get_read_name(src, record.read_name_mut(), l_read_name).map_err(ParseError::InvalidReadName)?;
    get_cigar(src, record.cigar_mut(), n_cigar_op).map_err(ParseError::InvalidCigar)?;
    get_sequence(src, record.sequence_mut(), l_seq).map_err(ParseError::InvalidSequence)?;
    get_quality_scores(src, record.quality_scores_mut(), l_seq)
        .map_err(ParseError::InvalidQualityScores)?;
    get_data(src, record.data_mut()).map_err(ParseError::InvalidData)?;

    cigar::resolve(header, record).map_err(ParseError::InvalidCigar)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_record_with_invalid_l_read_name() {
        let data = vec![
            0xff, 0xff, 0xff, 0xff, // ref_id = -1
            0xff, 0xff, 0xff, 0xff, // pos = -1
            0x00, // l_read_name = 0
        ];
        let mut src = &data[..];

        let header = sam::Header::default();
        let mut record = Record::default();

        assert!(matches!(
            decode_record(&mut src, &header, &mut record),
            Err(ParseError::InvalidReadName(_))
        ));
    }
}