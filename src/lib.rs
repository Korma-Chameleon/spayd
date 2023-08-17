//! This library implements taxt processing for the Short Payment Descriptor format
//! (SPAYD or SPD). This is a simple text format for requesting payments in
//! the Czech Republic and Slovakia. It can encode details of the payee,
//! destination account (IBAN), amount etc.

#[cfg(feature = "crc32")]
mod crc32;
mod parser;
mod spayd;

#[cfg(feature = "crc32")]
pub use crate::crc32::{Crc32Ok, Crc32Result};
pub use crate::parser::parse_spayd;
pub use crate::spayd::{Spayd, SpaydVersion};
