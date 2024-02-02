use std::io;

use noodles_vcf as vcf;

use crate::header::string_maps::StringStringMap;

/// BCF record genotypes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Genotypes<'a> {
    buf: &'a [u8],
    sample_count: usize,
    format_count: usize,
}

impl<'a> Genotypes<'a> {
    pub(super) fn new(buf: &'a [u8], sample_count: usize, format_count: usize) -> Self {
        Self {
            buf,
            sample_count,
            format_count,
        }
    }

    /// Converts BCF record genotypes to VCF record genotypes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bcf::{header::string_maps::StringMap, record::Genotypes};
    /// use noodles_vcf as vcf;
    ///
    /// let bcf_genotypes = Genotypes::default();
    ///
    /// let header = vcf::Header::default();
    /// let string_maps = StringMap::default();
    /// let vcf_genotypes = bcf_genotypes.try_into_vcf_record_genotypes(&header, &string_maps)?;
    ///
    /// assert!(vcf_genotypes.is_empty());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn try_into_vcf_record_genotypes(
        &self,
        header: &vcf::Header,
        string_map: &StringStringMap,
    ) -> io::Result<vcf::record::Genotypes> {
        use crate::record::codec::decoder::read_genotypes;

        if self.is_empty() {
            return Ok(vcf::record::Genotypes::default());
        }

        let mut reader = self.buf;

        let genotypes = read_genotypes(
            &mut reader,
            header.formats(),
            string_map,
            self.len(),
            self.format_count(),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(genotypes)
    }

    /// Returns the number of samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Genotypes;
    /// let genotypes = Genotypes::default();
    /// assert_eq!(genotypes.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.sample_count
    }

    /// Returns whether there are any samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Genotypes;
    /// let genotypes = Genotypes::default();
    /// assert!(genotypes.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of fields per sample.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf::record::Genotypes;
    /// let genotypes = Genotypes::default();
    /// assert_eq!(genotypes.format_count(), 0);
    /// ```
    pub fn format_count(&self) -> usize {
        self.format_count
    }
}

impl<'a> AsRef<[u8]> for Genotypes<'a> {
    fn as_ref(&self) -> &[u8] {
        self.buf
    }
}
