mod bounds;

pub(crate) use self::bounds::Bounds;
use super::{Attributes, Position, Strand};

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Fields {
    pub(crate) buf: String,
    pub(crate) bounds: Bounds,
}

impl Fields {
    pub fn reference_sequence_name(&self) -> &str {
        &self.buf[self.bounds.reference_sequence_name_range()]
    }

    pub fn source(&self) -> &str {
        &self.buf[self.bounds.source_range()]
    }

    pub fn ty(&self) -> &str {
        &self.buf[self.bounds.type_range()]
    }

    pub fn start(&self) -> Position<'_> {
        let buf = &self.buf[self.bounds.start_range()];
        Position::new(buf)
    }

    pub fn end(&self) -> Position<'_> {
        let buf = &self.buf[self.bounds.end_range()];
        Position::new(buf)
    }

    pub fn score(&self) -> &str {
        &self.buf[self.bounds.score_range()]
    }

    pub fn strand(&self) -> Strand<'_> {
        let buf = &self.buf[self.bounds.strand_range()];
        Strand::new(buf)
    }

    pub fn phase(&self) -> &str {
        &self.buf[self.bounds.phase_range()]
    }

    pub fn attributes(&self) -> Attributes<'_> {
        const MISSING: &str = ".";

        match &self.buf[self.bounds.attributes_range()] {
            MISSING => Attributes::new(""),
            buf => Attributes::new(buf),
        }
    }
}

impl Default for Fields {
    fn default() -> Self {
        Self {
            buf: String::from("...11...."),
            bounds: Bounds::default(),
        }
    }
}
