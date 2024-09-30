use anyhow::{anyhow, Result};
use reqwest::blocking::{Client, Response};
use serde::Deserialize;

const CROSSREF_WORKS_URL: &str = "https://api.crossref.org/works";

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
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
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
        let crossref = CrossRef::new();
        let work = crossref.query_work("richard feynman", "room at the bottom")?;
        assert_eq!(work.doi, "10.1201/9780429500459-7");

        Ok(())
    }
}
