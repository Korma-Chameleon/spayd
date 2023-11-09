//! This library implements text processing for the Short Payment Descriptor format
//! (SPAYD or SPD). This is a simple text format for requesting payments in
//! Czechia and the Slovak Republic. It can encode details of the payee,
//! destination account (IBAN), amount etc.
//!
//! Parsing SPAYD text:
//! ```
//! use spayd::{Spayd, fields};
//!
//! let payment: Spayd = "SPD*1.0*ACC:CZ1355000000000000222885*AM:250.00*CC:CZK".parse().unwrap();
//! let account = payment.field(fields::ACCOUNT).unwrap();
//! let amount = payment.field(fields::AMOUNT).unwrap();
//! let currency = payment.field(fields::CURRENCY).unwrap();
//!
//! println!("Please pay {}{} to account {}", amount, currency, account);
//! ```
//!
//! Creatig a SPAYD:
//! ```
//! use spayd::{Spayd, fields};
//!
//! let account = "CZ1355000000000000222885";
//! let amount = "250.00";
//! let currency = "CZK";
//!
//! let mut payment = Spayd::empty_v1_0();
//! payment.set_field(fields::ACCOUNT, account);
//! payment.set_field(fields::AMOUNT, amount);
//! payment.set_field(fields::CURRENCY, currency);
//!
//! println!("{}", payment.to_string());
//! ```

mod convert;
#[cfg(feature = "crc32")]
mod crc32;
mod error;
pub mod fields;
mod iban_bic;
mod parser;
mod spayd;

pub use crate::convert::*;
#[cfg(feature = "crc32")]
pub use crate::crc32::{Crc32Ok, Crc32Result};
pub use crate::error::SpaydError;
pub use crate::iban_bic::*;
pub use crate::spayd::*;
