//! Prints a BCF file in the VCF format.
//!
//! The result matches the output of `bcftools view <src>`.

use std::env;

use futures::TryStreamExt;
use noodles_bcf as bcf;
use noodles_vcf as vcf;
use tokio::{fs::File, io};

#[tokio::main]
async fn main() -> io::Result<()> {
    let src = env::args().nth(1).expect("missing src");

    let mut reader = File::open(src).await.map(bcf::r#async::io::Reader::new)?;
    let header = reader.read_header().await?;

    let mut records = reader.records();

    let mut writer = vcf::r#async::io::Writer::new(io::stdout());
    writer.write_header(&header).await?;

    while let Some(record) = records.try_next().await? {
        writer.write_variant_record(&header, &record).await?;
    }

    writer.shutdown().await?;

    Ok(())
}
