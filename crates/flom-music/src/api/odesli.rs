use std::collections::HashMap;

use flom_core::{FlomError, FlomResult};
use reqwest::Client;
use serde::Deserialize;

const API_BASE: &str = "https://api.song.link/v1-alpha.1/links";

#[derive(Debug, Clone)]
pub struct OdesliClient {
    client: Client,
    api_key: Option<String>,
    user_country: String,
}

impl OdesliClient {
    pub fn new(client: Client, api_key: Option<String>, user_country: impl Into<String>) -> Self {
        Self {
            client,
            api_key,
            user_country: user_country.into(),
        }
    }

    pub async fn fetch_links(&self, url: &str) -> FlomResult<OdesliResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("url", url.to_string()),
            ("userCountry", self.user_country.clone()),
        ];
        if let Some(key) = &self.api_key
            && !key.trim().is_empty() {
                params.push(("key", key.clone()));
            }

        let response = self
            .client
            .get(API_BASE)
            .query(&params)
            .header("Accept", "application/json")
            .header("User-Agent", "flom/0.1")
            .send()
            .await
            .map_err(|err| FlomError::Network(format!("odesli request failed: {err}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FlomError::Api(format!(
                "odesli error: status={status} body={body}"
            )));
        }

        response
            .json::<OdesliResponse>()
            .await
            .map_err(|err| FlomError::Parse(format!("odesli response parse failed: {err}")))
    }
}

#[derive(Debug, Deserialize)]
pub struct OdesliResponse {
    #[serde(rename = "entityUniqueId")]
    pub entity_unique_id: String,
    #[serde(rename = "pageUrl")]
    pub page_url: String,
    #[serde(rename = "linksByPlatform")]
    pub links_by_platform: HashMap<String, OdesliLink>,
    #[serde(rename = "entitiesByUniqueId")]
    pub entities_by_unique_id: HashMap<String, OdesliEntity>,
}

#[derive(Debug, Deserialize)]
pub struct OdesliLink {
    #[serde(rename = "entityUniqueId")]
    pub entity_unique_id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct OdesliEntity {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "albumName")]
    pub album_name: Option<String>,
    #[serde(rename = "apiProvider")]
    pub api_provider: Option<String>,
}

#[cfg(test)]
mod tests {
    use url::Url;

    #[test]
    fn test_validate_url_with_valid() {
        let result = Url::parse("https://example.com");
        assert!(result.is_ok(), "Valid https URL should parse successfully");

        let result = Url::parse("http://music.example.com/track/123");
        assert!(result.is_ok(), "Valid http URL should parse successfully");
    }

    #[test]
    fn test_validate_url_with_invalid() {
        let result = Url::parse("not-a-url");
        assert!(result.is_err(), "Invalid URL should fail to parse");

        let result = Url::parse("://no-scheme");
        assert!(result.is_err(), "URL without scheme should fail to parse");
    }
}
