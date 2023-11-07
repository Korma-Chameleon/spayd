use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use crate::{Spayd, SpaydError};
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "iban_validate")]
use iban::Iban;

const SPAYD_DATE_FMT: &str = "%Y%m%d";
const FIELD_DUE_DATE: &str = "DT";
const FIELD_ACC: &str = "ACC";

impl<'a> Spayd<'a> {
    /// Get the value of a field converted using the convert function
    fn field_converted<T, E, F>(&self, field: &str, convert: F) -> Result<T, SpaydError>
    where
        F: FnOnce(&str) -> Result<T, E>,
    {
        if let Some(text) = self.field(field) {
            convert(text).or(Err(SpaydError::ConvertError(text.into())))
        } else {
            Err(SpaydError::FieldMissing(field.into()))
        }
    }

    /// Set the due date from a Chrono NaiveDate
    fn set_field_converted<F, T>(&mut self, field: &'static str, value: T, convert: F)
    where
        F: FnOnce(T) -> String,
    {
        let text = convert(value);
        self.set_field(field, text);
    }
}

/// Separated IBAN and BIC strings from one of the account number fields
pub struct IbanBicText {
    /// International Bank Account Number
    pub iban: String,
    /// Bank Identifier Code (ISO 9362)
    pub bic: Option<String>,
}

impl<'a> Display for IbanBicText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(bic) = &self.bic {
            write!(f, "{}+{}", self.iban, bic)
        } else {
            write!(f, "{}", self.iban)
        }
    }
}

impl FromStr for IbanBicText {
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

impl<'a> Spayd<'a> {
    /// Get the account number as a separated IBAN and BIC
    pub fn acc_iban_bic_text(&self) -> Result<IbanBicText, SpaydError> {
        self.field_converted(FIELD_ACC, |text| text.parse())
    }

    /// Set the account IBAN from an Iban value
    pub fn set_acc_iban_bic_text(&mut self, iba_bic: &IbanBicText) {
        self.set_field_converted(FIELD_ACC, iba_bic, |iba_bic| iba_bic.to_string())
    }
}

/// Separated and parsed IBAN and BIC
#[cfg(feature = "iban_validate")]
#[derive(Debug, PartialEq, Eq)]
pub struct IbanBic {
    /// International Bank Account Number
    pub iban: Iban,
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
        let iban_bic = text.parse::<IbanBicText>()?;
        if let Ok(iban) = iban_bic.iban.parse() {
            Ok(Self {
                iban: iban,
                bic: iban_bic.bic,
            })
        } else {
            Err(Self::Err::ConvertError(text.to_owned()))
        }
    }
}

#[cfg(feature = "iban_validate")]
impl<'a> Spayd<'a> {
    /// Get the account number as an Iban
    pub fn acc_iban(&self) -> Result<IbanBic, SpaydError> {
        self.field_converted(FIELD_ACC, |text| text.parse())
    }

    /// Set the account IBAN from an Iban value
    pub fn set_acc_iban(&mut self, iban: &IbanBic) {
        self.set_field_converted(FIELD_ACC, iban, |iban| iban.to_string())
    }
}

#[cfg(feature = "chrono")]
impl<'a> Spayd<'a> {
    /// Get the due date as a Chrono NaiveDate
    pub fn due_date(&self) -> Result<NaiveDate, SpaydError> {
        self.field_converted(FIELD_DUE_DATE, |text| {
            NaiveDate::parse_from_str(text, SPAYD_DATE_FMT)
        })
    }

    /// Set the due date from a Chrono NaiveDate
    pub fn set_due_date(&mut self, date: &NaiveDate) {
        self.set_field_converted(FIELD_DUE_DATE, date, |date| {
            date.format(SPAYD_DATE_FMT).to_string()
        })
    }
}

#[cfg(feature = "iban_validate")]
#[cfg(test)]
mod iban_tests {
    use super::*;

    #[test]
    fn acc_no_bic() {
        let spayd = Spayd::new_v1_0(vec![("ACC", "CZ5855000000001265098001")]);
        assert_eq!(
            spayd.acc_iban(),
            Ok(IbanBic {
                iban: "CZ5855000000001265098001".parse().unwrap(),
                bic: None
            })
        )
    }

    #[test]
    fn acc_with_bic() {
        let spayd = Spayd::new_v1_0(vec![("ACC", "CZ5855000000001265098001+RZBCCZPP")]);
        assert_eq!(
            spayd.acc_iban(),
            Ok("CZ5855000000001265098001+RZBCCZPP".parse().unwrap())
        )
    }

    #[test]
    fn acc_incorrect_format() {
        let spayd = Spayd::new_v1_0(vec![("ACC", "INAVLID")]);
        assert_eq!(
            spayd.acc_iban(),
            Err(SpaydError::ConvertError("INAVLID".into()))
        )
    }
}

#[cfg(feature = "chrono")]
#[cfg(test)]
mod chrono_tests {
    use super::*;

    #[test]
    fn due_date_correct_format() {
        let spayd = Spayd::new_v1_0(vec![("DT", "20121231")]);
        assert_eq!(
            spayd.due_date(),
            Ok(NaiveDate::from_ymd_opt(2012, 12, 31).unwrap())
        )
    }

    #[test]
    fn due_date_incorrect_format() {
        let spayd = Spayd::new_v1_0(vec![("DT", "2012/12/31")]);
        assert_eq!(
            spayd.due_date(),
            Err(SpaydError::ConvertError("2012/12/31".into()))
        )
    }

    #[test]
    fn set_due_date_correct_format() {
        let mut spayd = Spayd::empty_v1_0();
        spayd.set_due_date(&NaiveDate::from_ymd_opt(2012, 12, 31).unwrap());
        assert_eq!(spayd.field("DT"), Some("20121231"))
    }
}
