use crate::error::SpaydError;
#[cfg(feature = "iban_validate")]
use iban::Iban;
use std::fmt::Display;
use std::{fmt::Formatter, str::FromStr};

/// Separated IBAN and BIC strings from one of the account number fields
#[derive(Debug, PartialEq, Eq)]
pub struct IbanBic {
    /// International Bank Account Number
    pub iban: String,
    /// Bank Identifier Code (ISO 9362)
    pub bic: Option<String>,
}

impl<'a> Display for IbanBic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(bic) = &self.bic {
            write!(f, "{}+{}", self.iban, bic)
        } else {
            write!(f, "{}", self.iban)
        }
    }
}

impl FromStr for IbanBic {
    type Err = SpaydError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        {
            if let Some((iban, bic)) = text.split_once('+') {
                Ok(Self {
                    iban: iban.to_owned(),
                    bic: Some(bic.to_owned()),
                })
            } else {
                Ok(Self {
                    iban: text.to_owned(),
                    bic: None,
                })
            }
        }
    }
}

impl IbanBic {
    /// Construct and IbanBic with only an IBAN and no BIC
    pub fn iban_only<T>(iban: T) -> Self
    where
        T: ToString,
    {
        Self {
            iban: iban.to_string(),
            bic: None,
        }
    }

    /// Construct and IbanBic with both an IBAN and a BIC
    pub fn iban_bic<T, U>(iban: T, bic: U) -> Self
    where
        T: ToString,
        U: ToString,
    {
        Self {
            iban: iban.to_string(),
            bic: Some(bic.to_string()),
        }
    }

    /// Parse the IBAN value into an Iban object from the iban_validate crate
    #[cfg(feature = "iban_validate")]
    pub fn parse_iban(&self) -> Result<Iban, SpaydError> {
        self.iban
            .parse()
            .or(Err(SpaydError::ConvertError(self.iban.to_owned())))
    }
}
