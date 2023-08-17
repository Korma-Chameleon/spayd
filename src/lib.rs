#[cfg(feature = "crc32")]
mod crc32;
mod parser;
mod spayd;

#[cfg(feature = "crc32")]
pub use crate::crc32::{Crc32Ok, Crc32Result};
pub use crate::parser::parse_spayd;
pub use crate::spayd::Spayd;
