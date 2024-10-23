use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceRecord {
    pub author: String,
    pub title: String,
}

pub trait RecordParser {
    fn parse(&mut self) -> Result<Vec<ResourceRecord>>;
}
