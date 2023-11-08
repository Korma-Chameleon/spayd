use std::str::FromStr;

use crate::{IbanBic, Spayd, SpaydError};

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[cfg(feature = "iso_currency")]
use iso_currency::Currency;

const SPAYD_DATE_FMT: &str = "%Y%m%d";

const FIELD_DUE_DATE: &str = "DT";
const FIELD_ACC: &str = "ACC";
const FIELD_CURRENCY: &str = "CC";

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
    fn set_field_converted<F, T, U>(&mut self, field: &'static str, value: T, convert: F)
    where
        F: FnOnce(T) -> U,
        U: ToString,
    {
        let text = convert(value);
        self.set_field(field, text.to_string());
    }

    /// Get the account number as a separated IBAN and BIC
    pub fn acc_iban_bic(&self) -> Result<IbanBic, SpaydError> {
        self.field_converted(FIELD_ACC, IbanBic::from_str)
    }

    /// Set the account IBAN from an Iban value
    pub fn set_acc_iban_bic(&mut self, iba_bic: &IbanBic) {
        self.set_field_converted(FIELD_ACC, iba_bic, IbanBic::to_string)
    }

    /// Get the due date as a Chrono NaiveDate
    #[cfg(feature = "chrono")]
    pub fn due_date(&self) -> Result<NaiveDate, SpaydError> {
        self.field_converted(FIELD_DUE_DATE, |text| {
            NaiveDate::parse_from_str(text, SPAYD_DATE_FMT)
        })
    }

    /// Set the due date from a Chrono NaiveDate
    #[cfg(feature = "chrono")]
    pub fn set_due_date(&mut self, date: &NaiveDate) {
        self.set_field_converted(FIELD_DUE_DATE, date, |date| {
            date.format(SPAYD_DATE_FMT).to_string()
        })
    }

    /// Get the currency as an ISO currency
    #[cfg(feature = "iso_currency")]
    pub fn currency(&self) -> Result<Currency, SpaydError> {
        self.field_converted(FIELD_CURRENCY, |currency| {
            Currency::from_code(currency).ok_or(())
        })
    }

    /// Set the currency using an ISO currency
    #[cfg(feature = "iso_currency")]
    pub fn set_currency(&mut self, currency: Currency) {
        self.set_field_converted(FIELD_CURRENCY, currency, Currency::code)
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
            spayd.acc_iban_bic(),
            Ok(IbanBic {
                iban: "CZ5855000000001265098001".to_owned(),
                bic: None
            })
        )
    }

    #[test]
    fn acc_with_bic() {
        let spayd = Spayd::new_v1_0(vec![("ACC", "CZ5855000000001265098001+RZBCCZPP")]);
        assert_eq!(
            spayd.acc_iban_bic(),
            Ok(IbanBic {
                iban: "CZ5855000000001265098001".to_owned(),
                bic: Some("RZBCCZPP".to_owned())
            })
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
