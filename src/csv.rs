use anyhow::{Error, Result};
use csv::Reader;
use std::{fs::File, path::Path};

use crate::record::{RecordParser, ResourceRecord};

pub struct CsvParser {
    reader: Reader<File>,
}

impl CsvParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            reader: csv::Reader::from_reader(File::open(path)?),
        })
    }
}

impl RecordParser for CsvParser {
    fn parse(&mut self) -> Result<Vec<ResourceRecord>> {
        self.reader
            .deserialize::<ResourceRecord>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() -> Result<()> {
        let mut csv_parser = CsvParser::new("test_data/test.csv")?;
        let records = csv_parser.parse()?;
        assert_eq!(records[0].author, "Richard Feynman");
        assert_eq!(records[0].title, "Room at the bottom");
        assert_eq!(records[2].author, "Richard Feynman");
        assert_eq!(records[2].title, "What is science?");

        Ok(())
    }
}
