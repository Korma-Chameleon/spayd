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
//!
//! assert_eq!(payment.field(fields::ACCOUNT), Some("CZ1355000000000000222885"));
//! assert_eq!(payment.field(fields::AMOUNT), Some("250.00"));
//! assert_eq!(payment.field(fields::CURRENCY), Some("CZK"));
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
//! assert_eq!(payment.to_string(), "SPD*1.0*ACC:CZ1355000000000000222885*AM:250.00*CC:CZK")
//! ```
//!
//! This crate also provides features (chrono, iban_validate, iso_currency, rust_decimal) for
//! optional conversions to/from commonly used types.
//! ```
//! use spayd::{Spayd, fields};
//! use iban::Iban;
//! use chrono::NaiveDate;
//! use rust_decimal::Decimal;
//! use iso_currency::Currency;
//!
//! let account: Iban = "CZ1355000000000000222885".parse().unwrap();
//! let amount = Decimal::new(250, 0);;
//! let currency = Currency::CZK;
//! let due_date = NaiveDate::from_ymd_opt(2023, 10, 31).unwrap();
//!
//! let mut payment = Spayd::empty_v1_0();
//! payment.set_account(account);
//! payment.set_amount(&amount);
//! payment.set_currency(currency);
//! payment.set_due_date(&due_date);
//!
//! assert_eq!(payment.account().unwrap().to_iban(), Ok(account));
//! assert_eq!(payment.amount(), Ok(amount));
//! assert_eq!(payment.currency(), Ok(currency));
//! assert_eq!(payment.due_date(), Ok(due_date));
//! ```
//!

mod convert;
#[cfg(feature = "crc32")]
mod crc32;
mod error;
/// Constants for the standard field names.
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
