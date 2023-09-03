use nom::error::Error as NomError;
use thiserror::Error;

/// Errors encountered when parsing and validating SPAYD values.
#[derive(Error, Debug, PartialEq)]
pub enum SpaydError {
    /// Parsing failed. The supplied text is in an incorrect format.
    #[error("couldn't parse text: {0}")]
    ParseError(#[from] NomError<String>),
    /// A field required by the SPAYD standard is missing. The field name
    /// is supplied in the error. In SPAYD version 1.0, only the ACC field
    /// is required.
    #[error("the required field '{0}' is missing")]
    RequiredFieldMissing(String),
    /// The CRC32 checksum failed. The SPAYD value is probably incorrect
    /// or has been corrupted.
    #[cfg(feature = "crc32")]
    #[error("the data doesn't match the CRC32 checksum")]
    Crc32Failed,
}

impl From<NomError<&str>> for SpaydError {
    fn from(value: NomError<&str>) -> Self {
        Self::ParseError(NomError {
            input: value.input.to_owned(),
            code: value.code,
        })
    }
}
