use std::io::{self, Write};

use crate::variant::record::samples::keys::key;

pub(super) fn write_keys<'a, W, I>(writer: &mut W, keys: I) -> io::Result<()>
where
    W: Write,
    I: Iterator<Item = io::Result<&'a str>>,
{
    const DELIMITER: &[u8] = b":";

    for (i, result) in keys.enumerate() {
        let key = result?;

        if i > 0 {
            if key == key::GENOTYPE {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "GT must be first series",
                ));
            }

            writer.write_all(DELIMITER)?;
        }

        write_key(writer, key)?;
    }

    Ok(())
}

fn write_key<W>(writer: &mut W, key: &str) -> io::Result<()>
where
    W: Write,
{
    if is_valid(key) {
        writer.write_all(key.as_bytes())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid genotype field key",
        ))
    }
}

// § 1.6.2 "Genotype fields" (2023-08-23): "`^[A-Za-z_][0-9A-Za-z_.]*$`".
fn is_valid(s: &str) -> bool {
    fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '.')
    }

    let mut chars = s.chars();

    let is_valid_first_char = chars
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or_default();

    is_valid_first_char && chars.all(is_valid_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_keys() -> io::Result<()> {
        let mut buf = Vec::new();

        buf.clear();
        let keys = [Ok("GT")];
        write_keys(&mut buf, keys.into_iter())?;
        assert_eq!(buf, b"GT");

        buf.clear();
        let keys = [Ok("GT"), Ok("GQ")];
        write_keys(&mut buf, keys.into_iter())?;
        assert_eq!(buf, b"GT:GQ");

        buf.clear();
        let keys = [Ok("GQ")];
        write_keys(&mut buf, keys.into_iter())?;
        assert_eq!(buf, b"GQ");

        buf.clear();
        let keys = [Ok("GQ"), Ok("GT")];
        assert!(matches!(
            write_keys(&mut buf, keys.into_iter()),
            Err(e) if e.kind() == io::ErrorKind::InvalidInput
        ));

        Ok(())
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid("GT"));
        assert!(is_valid("PSL"));

        assert!(!is_valid(""));
        assert!(!is_valid("G T"));
        assert!(!is_valid("1000G"));
    }
}
