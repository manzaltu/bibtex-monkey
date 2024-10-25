use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use clap::{crate_name, crate_version};
use reqwest::blocking::{Client, Response};
use serde::Deserialize;

const CROSSREF_WORKS_URL: &str = "https://api.crossref.org/works";
const CROSSREF_BIBTEX_URL_SUFFIX: &str = "/transform/application/x-bibtex";
const USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

#[derive(Deserialize)]
struct CrossRefResponse {
    status: String,
    #[serde(flatten)]
    message: CrossRefMessage,
}

#[derive(Deserialize)]
#[serde(tag = "message-type", content = "message")]
enum CrossRefMessage {
    #[serde(rename = "work-list")]
    WorksMessage(CrossRefGenericMessage<CrossRefWork>),
}

#[derive(Deserialize)]
struct CrossRefGenericMessage<T> {
    items: Vec<T>,
}

#[derive(Deserialize)]
pub struct CrossRefWork {
    #[serde(rename = "DOI")]
    pub doi: String,
}

pub struct CrossRef {
    client: Client,
}

impl CrossRef {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::blocking::Client::builder()
                .user_agent(USER_AGENT)
                .build()?,
        })
    }

    pub fn query_work(&self, author: &str, title: &str) -> Result<CrossRefWork> {
        let message = self.query_works_message(author, title)?;
        let work = message
            .items
            .into_iter()
            .next()
            .ok_or(anyhow!("No work found"))?;

        Ok(work)
    }

    pub fn download_work_bibtex<P: AsRef<Path>>(&self, doi: &str, path: P) -> Result<()> {
        let bibtex_data = self.get_work_bibtex_data(doi)?;
        fs::write(path, bibtex_data)?;

        Ok(())
    }

    fn query_works_message(
        &self,
        author: &str,
        title: &str,
    ) -> Result<CrossRefGenericMessage<CrossRefWork>> {
        let request = self.client.get(CROSSREF_WORKS_URL);
        let response = request
            .query(&[("query.title", title), ("query.author", author)])
            .send()?;

        let message = Self::parse_json_response(response)?;

        let CrossRefMessage::WorksMessage(works) = message;
        Ok(works)
    }

    fn get_work_bibtex_data(&self, doi: &str) -> Result<String> {
        let url = CROSSREF_WORKS_URL.to_string() + "/" + doi + CROSSREF_BIBTEX_URL_SUFFIX;
        let response = self.client.get(url).send()?;

        Ok(response.text()?)
    }

    fn parse_json_response(response: Response) -> Result<CrossRefMessage> {
        let parsed_response = response.json::<CrossRefResponse>()?;
        if parsed_response.status != "ok" {
            return Err(anyhow!("Crossref server error: {}", parsed_response.status));
        }

        Ok(parsed_response.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_query() -> Result<()> {
        let crossref = CrossRef::new()?;
        let work = crossref.query_work("richard feynman", "room at the bottom")?;
        assert_eq!(work.doi, "10.1201/9780429500459-7");

        Ok(())
    }

    #[test]
    fn test_get_work_bibtex() -> Result<()> {
        let crossref = CrossRef::new()?;
        let data = crossref.get_work_bibtex_data("10.1201/9780429500459-7")?;
        assert_eq!(data, " @inbook{Feynman, title={There’s Plenty of Room at the Bottom}, ISBN={9780429500459}, url={http://dx.doi.org/10.1201/9780429500459-7}, DOI={10.1201/9780429500459-7}, booktitle={Feynman and Computation}, publisher={CRC Press}, author={Feynman, Richard}, pages={63–76} }\n");

        Ok(())
    }
}
