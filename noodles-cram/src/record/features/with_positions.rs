use noodles_core::Position;

use crate::record::Feature;

pub struct WithPositions<'a, I>
where
    I: Iterator<Item = &'a Feature>,
{
    iter: I,
    reference_position: Position,
    read_position: Position,
}

impl<'a, I> WithPositions<'a, I>
where
    I: Iterator<Item = &'a Feature>,
{
    pub fn new(iter: I, alignment_start: Position) -> Self {
        Self {
            iter,
            reference_position: alignment_start,
            read_position: Position::MIN,
        }
    }

    /// Returns the current reference position and read position.
    ///
    /// These are 1-based.
    pub fn positions(&self) -> (Position, Position) {
        (self.reference_position, self.read_position)
    }
}

impl<'a, I> Iterator for WithPositions<'a, I>
where
    I: Iterator<Item = &'a Feature>,
{
    type Item = ((Position, Position), I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let feature = self.iter.next()?;

        let feature_position = usize::from(feature.position());
        let match_len = feature_position - usize::from(self.read_position);

        self.reference_position = self
            .reference_position
            .checked_add(match_len)
            .expect("attempt to add with overflow");

        self.read_position = self
            .read_position
            .checked_add(match_len)
            .expect("attempt to add with overflow");

        let (reference_position_delta, read_position_delta) = match feature {
            Feature::Bases(_, bases) => (bases.len(), bases.len()),
            Feature::ReadBase(..) => (1, 1),
            Feature::Substitution(..) => (1, 1),
            Feature::Insertion(_, bases) => (0, bases.len()),
            Feature::Deletion(_, len) => (*len as usize, 0),
            Feature::InsertBase(..) => (0, 1),
            Feature::ReferenceSkip(_, len) => (*len as usize, 0),
            Feature::SoftClip(_, bases) => (0, bases.len()),
            Feature::Padding(..) => (0, 0),
            Feature::HardClip(..) => (0, 0),
            _ => todo!("unhandled feature: {:?}", feature),
        };

        let positions = self.positions();

        self.reference_position = self
            .reference_position
            .checked_add(reference_position_delta)
            .expect("attempt to add with overflow");

        self.read_position = self
            .read_position
            .checked_add(read_position_delta)
            .expect("attempt to add with overflow");

        Some((positions, feature))
    }
}
