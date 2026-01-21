mod error;
mod result;

pub use error::{FlomError, FlomResult};
pub use result::{ConversionResult, MediaInfo};

pub fn validate_url(url: &str) -> FlomResult<()> {
    url::Url::parse(url).map_err(|err| FlomError::InvalidInput(format!("invalid url: {err}")))?;
    Ok(())
}

pub trait Converter {
    fn convert(&self, input: &str, target: Option<&str>) -> FlomResult<ConversionResult>;
}
