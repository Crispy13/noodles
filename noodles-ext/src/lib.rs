use noodles_bam::Record;
use noodles_sam::{self as sam, alignment::RecordBuf};

pub trait RecordExt {
    fn name_as_str(&self) -> Option<Result<&str, std::str::Utf8Error>>;

    fn contig<'h>(
        &self,
        header: &'h sam::Header,
    ) -> Option<Result<std::borrow::Cow<'h, str>, std::io::Error>>;
}

impl RecordExt for Record {
    fn name_as_str(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.name().map(|n| std::str::from_utf8(n.as_bytes()))
    }

    fn contig<'h>(
        &self,
        header: &'h sam::Header,
    ) -> Option<Result<std::borrow::Cow<'h, str>, std::io::Error>> {
        match self.reference_sequence_id() {
            Some(r) => match r {
                Ok(i) => header
                    .reference_sequences()
                    .get_index(i)
                    .map(|c| Ok(String::from_utf8_lossy(c.0))),
                Err(err) => Some(Err(err)),
            },
            None => None,
        }
    }
}

impl RecordExt for RecordBuf {
    fn name_as_str(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.name().map(|n| std::str::from_utf8(n.as_ref()))
    }

    fn contig<'h>(
        &self,
        header: &'h sam::Header,
    ) -> Option<Result<std::borrow::Cow<'h, str>, std::io::Error>> {
        self.reference_sequence(header)
            .map(|v| v.map(|v| String::from_utf8_lossy(v.0)))
    }
}

// struct CPU<'a>(&'a [u8]);

// impl<'a> CPU<'a> {
//     fn get(&self) -> &'a [u8] {
//         self.0
//     }
// }

// impl<'a> AsRef<[u8]> for CPU<'a> {
//     fn as_ref(&self) -> &'a [u8] {
//         self.0
//     }
// }

// fn ret<'a>(a: &'a [u8]) -> &'a [u8] {
//     CPU(a).as_ref()
// }
