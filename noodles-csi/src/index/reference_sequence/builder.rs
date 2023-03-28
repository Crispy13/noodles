use std::collections::HashMap;

use noodles_bgzf as bgzf;
use noodles_core::Position;

use super::{
    bin::{self, Chunk},
    Bin, Metadata, ReferenceSequence,
};

#[derive(Debug)]
pub struct Builder {
    bin_builders: HashMap<usize, bin::Builder>,
    start_position: bgzf::VirtualPosition,
    end_position: bgzf::VirtualPosition,
}

impl Builder {
    pub fn add_record(
        &mut self,
        min_shift: u8,
        depth: u8,
        start: Position,
        end: Position,
        chunk: Chunk,
    ) {
        self.update_bins(min_shift, depth, start, end, chunk);
        self.update_metadata(chunk);
    }

    pub fn build(self) -> ReferenceSequence {
        if self.bin_builders.is_empty() {
            return ReferenceSequence::new(Vec::new(), None);
        }

        let bins = self.bin_builders.into_values().map(|b| b.build()).collect();
        let metadata = Metadata::new(self.start_position, self.end_position, 0, 0);
        ReferenceSequence::new(bins, Some(metadata))
    }

    fn update_bins(
        &mut self,
        min_shift: u8,
        depth: u8,
        start: Position,
        end: Position,
        chunk: Chunk,
    ) {
        use super::reg2bin;

        let bin_id = reg2bin(start, end, min_shift, depth);

        let builder = self
            .bin_builders
            .entry(bin_id)
            .or_insert_with(|| Bin::builder().set_id(bin_id));

        builder.add_chunk(chunk);
    }

    fn update_metadata(&mut self, chunk: Chunk) {
        // TODO: Update mapped and unmapped record counts.

        self.start_position = self.start_position.min(chunk.start());
        self.end_position = self.end_position.max(chunk.end());
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            bin_builders: HashMap::new(),
            start_position: bgzf::VirtualPosition::MAX,
            end_position: bgzf::VirtualPosition::MIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() -> Result<(), Box<dyn std::error::Error>> {
        const MIN_SHIFT: u8 = 14;
        const DEPTH: u8 = 5;

        let mut builder = Builder::default();

        builder.add_record(
            MIN_SHIFT,
            DEPTH,
            Position::try_from(2)?,
            Position::try_from(5)?,
            Chunk::new(
                bgzf::VirtualPosition::from(55),
                bgzf::VirtualPosition::from(89),
            ),
        );

        builder.add_record(
            MIN_SHIFT,
            DEPTH,
            Position::try_from(8)?,
            Position::try_from(13)?,
            Chunk::new(
                bgzf::VirtualPosition::from(89),
                bgzf::VirtualPosition::from(144),
            ),
        );

        let actual = builder.build();

        let expected = ReferenceSequence::new(
            vec![Bin::new(
                4681,
                bgzf::VirtualPosition::from(55),
                vec![Chunk::new(
                    bgzf::VirtualPosition::from(55),
                    bgzf::VirtualPosition::from(144),
                )],
            )],
            Some(Metadata::new(
                bgzf::VirtualPosition::from(55),
                bgzf::VirtualPosition::from(144),
                0,
                0,
            )),
        );

        assert_eq!(actual, expected);

        Ok(())
    }
}