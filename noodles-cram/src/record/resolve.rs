use noodles_fasta as fasta;

use crate::{
    compression_header::preservation_map::substitution_matrix::Base,
    compression_header::SubstitutionMatrix, Feature,
};

pub fn resolve_bases(
    reference_sequence_record: &fasta::Record,
    substitution_matrix: &SubstitutionMatrix,
    features: &[Feature],
    alignment_start: i32,
    read_len: usize,
) -> Vec<u8> {
    let mut buf = vec![b'-'; read_len];

    let mut ref_pos = (alignment_start - 1) as usize;
    let mut read_pos = 0;

    for feature in features {
        match feature {
            Feature::Substitution(position, code) => {
                let reference_sequence = reference_sequence_record.sequence();

                for _ in 0..(*position - 1) {
                    buf[read_pos] = reference_sequence[ref_pos];
                    ref_pos += 1;
                    read_pos += 1;
                }

                let base = reference_sequence[ref_pos];

                let reference_base = match base {
                    b'A' => Base::A,
                    b'C' => Base::C,
                    b'G' => Base::G,
                    b'T' => Base::T,
                    b'N' => Base::N,
                    _ => panic!("unknown base value: {:?}", base),
                };

                let read_base = match substitution_matrix.get(reference_base, *code) {
                    Base::A => b'A',
                    Base::C => b'C',
                    Base::G => b'G',
                    Base::T => b'T',
                    Base::N => b'N',
                };

                buf[read_pos] = read_base;

                ref_pos += 1;
                read_pos += 1;
            }
            _ => todo!("resolve_bases: {:?}", feature),
        }
    }

    buf
}
