use flom_core::{ConversionResult, FlomError, FlomResult};

pub struct UrlConverter;

impl UrlConverter {
    pub fn convert(&self, input: &str, target: Option<&str>) -> FlomResult<ConversionResult> {
        let target = target.ok_or_else(|| {
            FlomError::UnsupportedInput("target is required for url conversion".to_string())
        })?;
        Ok(ConversionResult {
            source_url: input.to_string(),
            target_url: Some(target.to_string()),
            source_platform: None,
            target_platform: None,
            source_info: None,
            target_info: None,
            warning: Some("url conversion is not implemented yet".to_string()),
        })
    }
}
