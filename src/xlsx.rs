use anyhow::{Error, Result};
use std::path::Path;

use calamine::{open_workbook, Data, Range, RangeDeserializerBuilder, Reader, Xlsx};

use crate::record::{RecordParser, ResourceRecord};

pub struct XlsxParser {
    range: Range<Data>,
}

impl XlsxParser {
    pub fn new<P: AsRef<Path>>(path: P, worksheet: &str) -> Result<Self> {
        Ok(Self {
            range: open_workbook::<Xlsx<_>, _>(path)?.worksheet_range(worksheet)?,
        })
    }
}

impl RecordParser for XlsxParser {
    fn parse(&mut self) -> anyhow::Result<Vec<crate::record::ResourceRecord>> {
        RangeDeserializerBuilder::new()
            .from_range(&self.range)?
            .collect::<Result<Vec<ResourceRecord>, _>>()
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() -> Result<()> {
        let mut xlsx_parser = XlsxParser::new("test_data/test.xlsx", "Sheet1")?;
        let records = xlsx_parser.parse()?;
        assert_eq!(records[0].author, "Richard Feynman");
        assert_eq!(records[0].title, "Room at the bottom");
        assert_eq!(records[2].author, "Richard Feynman");
        assert_eq!(records[2].title, "What is science?");

        Ok(())
    }
}
