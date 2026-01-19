mod error;
mod result;

pub use error::{FlomError, FlomResult};
pub use result::{ConversionResult, MediaInfo};

pub trait Converter {
    fn convert(&self, input: &str, target: Option<&str>) -> FlomResult<ConversionResult>;
}
