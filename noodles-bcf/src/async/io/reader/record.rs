use crate::Record;
use tokio::io::{self, AsyncRead, AsyncReadExt};

pub(super) async fn read_record<R>(reader: &mut R, record: &mut Record) -> io::Result<usize>
where
    R: AsyncRead + Unpin,
{
    use crate::io::reader::record::read_site;

    let l_shared = match reader.read_u32_le().await {
        Ok(n) => usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(0),
        Err(e) => return Err(e),
    };

    let l_indiv = reader.read_u32_le().await.and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    let site_buf = record.fields_mut().site_buf_mut();
    site_buf.resize(l_shared, 0);
    reader.read_exact(site_buf).await?;

    let buf = site_buf.clone();
    let mut buf_reader = &buf[..];
    read_site(&mut buf_reader, record)?;

    record.fields_mut().index()?;

    let samples_buf = record.fields_mut().samples_buf_mut();
    samples_buf.resize(l_indiv, 0);
    reader.read_exact(samples_buf).await?;

    Ok(l_shared + l_indiv)
}

#[cfg(test)]
mod tests {
    use noodles_vcf::{
        record::{
            genotypes::{self, sample::Value as GenotypeFieldValue, Keys},
            info, AlternateBases, Filters as VcfFilters, Genotypes as VcfGenotypes, Position,
        },
        variant::record::info::field::Value as InfoFieldValue,
    };

    use super::*;
    use crate::header::StringMaps;

    #[tokio::test]
    async fn test_read_record() -> Result<(), Box<dyn std::error::Error>> {
        use crate::io::reader::record::tests::{DATA, RAW_HEADER};

        let header = RAW_HEADER.parse()?;
        let string_maps: StringMaps = RAW_HEADER.parse()?;

        let mut reader = &DATA[..];
        let mut record = Record::default();
        read_record(&mut reader, &mut record).await?;

        assert_eq!(record.chromosome_id()?, 1);
        assert_eq!(record.position()?, Position::from(101));
        assert_eq!(record.rlen()?, 1);
        assert_eq!(record.quality_score()?, Some(30.1));
        assert_eq!(record.ids().as_ref(), b"rs123");
        assert_eq!(record.reference_bases().as_ref(), b"A");
        assert_eq!(
            record.alternate_bases(),
            &AlternateBases::from(vec![String::from("C")])
        );

        assert_eq!(
            record
                .filters()
                .try_into_vcf_record_filters(string_maps.strings())?,
            Some(VcfFilters::Pass),
        );

        // info

        let mut iter = record.info().iter(&header, string_maps.strings());
        assert!(matches!(
            iter.next(),
            Some(Ok(("HM3", Some(InfoFieldValue::Flag))))
        ));
        assert!(matches!(
            iter.next(),
            Some(Ok((
                info::field::key::ALLELE_COUNT,
                Some(InfoFieldValue::Integer(3))
            )))
        ));
        assert!(matches!(
            iter.next(),
            Some(Ok((
                info::field::key::TOTAL_ALLELE_COUNT,
                Some(InfoFieldValue::Integer(6))
            )))
        ));
        assert!(matches!(
            iter.next(),
            Some(Ok((
                info::field::key::ANCESTRAL_ALLELE,
                Some(InfoFieldValue::String("C"))
            )))
        ));
        assert!(iter.next().is_none());

        // genotypes

        let actual = record
            .genotypes()?
            .try_into_vcf_record_genotypes(&header, string_maps.strings())?;

        let expected = VcfGenotypes::new(
            Keys::try_from(vec![
                String::from(genotypes::keys::key::GENOTYPE),
                String::from(genotypes::keys::key::CONDITIONAL_GENOTYPE_QUALITY),
                String::from(genotypes::keys::key::READ_DEPTH),
                String::from(genotypes::keys::key::READ_DEPTHS),
                String::from(genotypes::keys::key::ROUNDED_GENOTYPE_LIKELIHOODS),
            ])?,
            vec![
                vec![
                    Some(GenotypeFieldValue::from("0/0")),
                    Some(GenotypeFieldValue::from(10)),
                    Some(GenotypeFieldValue::from(32)),
                    Some(GenotypeFieldValue::from(vec![Some(32), Some(0)])),
                    Some(GenotypeFieldValue::from(vec![Some(0), Some(10), Some(100)])),
                ],
                vec![
                    Some(GenotypeFieldValue::from("0/1")),
                    Some(GenotypeFieldValue::from(10)),
                    Some(GenotypeFieldValue::from(48)),
                    Some(GenotypeFieldValue::from(vec![Some(32), Some(16)])),
                    Some(GenotypeFieldValue::from(vec![Some(10), Some(0), Some(100)])),
                ],
                vec![
                    Some(GenotypeFieldValue::from("1/1")),
                    Some(GenotypeFieldValue::from(10)),
                    Some(GenotypeFieldValue::from(64)),
                    Some(GenotypeFieldValue::from(vec![Some(0), Some(64)])),
                    Some(GenotypeFieldValue::from(vec![Some(100), Some(10), Some(0)])),
                ],
            ],
        );

        assert_eq!(actual, expected);

        Ok(())
    }
}