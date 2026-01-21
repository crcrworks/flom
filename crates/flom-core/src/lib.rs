mod error;
mod result;

pub use error::{FlomError, FlomResult};
pub use result::{ConversionResult, MediaInfo};

pub fn validate_url(url: &str) -> FlomResult<()> {
    url::Url::parse(url).map_err(|err| FlomError::InvalidInput(format!("invalid url: {err}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_url;
    use crate::FlomError;

    #[test]
    fn test_validate_url_valid_https() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://music.example.com/track/123").is_ok());
    }

    #[test]
    fn test_validate_url_valid_http() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("http://music.example.com/album/456").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        let result = validate_url("not-a-url");
        assert!(matches!(result, Err(FlomError::InvalidInput(_))));
    }

    #[test]
    fn test_validate_url_error_message() {
        let result = validate_url("://no-scheme");
        match result {
            Err(FlomError::InvalidInput(msg)) => assert!(msg.contains("invalid url")),
            _ => panic!("Expected InvalidInput error"),
        }
    }
}
