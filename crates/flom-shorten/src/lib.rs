use flom_core::{FlomError, FlomResult, validate_url};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ShortenClient {
    client: Client,
}

impl ShortenClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("flom/0.1")
            .build()
            .expect("failed to build http client");
        Self { client }
    }

    pub async fn shorten(&self, input: &str) -> FlomResult<String> {
        validate_url(input)?;
        let response = self
            .client
            .get("https://is.gd/create.php")
            .query(&[("format", "json"), ("url", input)])
            .send()
            .await
            .map_err(|err| FlomError::Network(format!("shorten request failed: {err}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FlomError::Api(format!(
                "shorten error: status={status} body={body}"
            )));
        }

        let payload = response
            .json::<ShortenResponse>()
            .await
            .map_err(|err| FlomError::Parse(format!("shorten response parse failed: {err}")))?;

        if let Some(error_message) = payload.errormessage {
            return Err(FlomError::Api(error_message));
        }

        payload
            .shorturl
            .ok_or_else(|| FlomError::Api("shorten response missing shorturl".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct ShortenResponse {
    shorturl: Option<String>,
    errormessage: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_with_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://music.example.com/track/123").is_ok());
    }

    #[test]
    fn test_validate_url_with_invalid() {
        let result = validate_url("not-a-url");
        assert!(result.is_err());
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error"),
        }

        let result = validate_url("://no-scheme");
        assert!(result.is_err());
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_shorten_validate_url() {
        let client = ShortenClient::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async { client.shorten("invalid-url-without-scheme").await });

        assert!(result.is_err());
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error from validate_url, not Network/Api error"),
        }
    }

    #[test]
    fn test_shorten_error_handling() {
        let client = ShortenClient::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async { client.shorten("not-a-url").await });
        assert!(matches!(result, Err(FlomError::InvalidInput(_))));

        let result = rt.block_on(async { client.shorten("https://").await });
        assert!(result.is_err());
    }
}
