use std::str::FromStr;

use crate::{IbanBic, Spayd, SpaydError};

#[cfg(feature = "chrono")]
use chrono::NaiveDate;

#[cfg(feature = "iso_currency")]
use iso_currency::Currency;

#[cfg(feature = "rust_decimal")]
use rust_decimal::Decimal;

const SPAYD_DATE_FMT: &str = "%Y%m%d";

const FIELD_DUE_DATE: &str = "DT";
const FIELD_ACCOUNT: &str = "ACC";
const FIELD_ALTERNATIVE_ACCOUNTS: &str = "ALT-ACC";
const FIELD_AMOUNT: &str = "AM";
const FIELD_CURRENCY: &str = "CC";

impl Spayd {
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
    pub fn account(&self) -> Result<IbanBic, SpaydError> {
        self.field_converted(FIELD_ACCOUNT, IbanBic::from_str)
    }

    /// Set the account IBAN from an IBAN and BIC
    pub fn set_account(&mut self, account: &IbanBic) {
        self.set_field_converted(FIELD_ACCOUNT, account, IbanBic::to_string)
    }

    /// Get alternative account numbers
    pub fn alternative_accounts(&self) -> Result<Vec<IbanBic>, SpaydError> {
        self.field_converted(FIELD_ALTERNATIVE_ACCOUNTS, |text| {
            text.split(',').map(IbanBic::from_str).collect()
        })
    }

    /// Set alternative account numbers
    pub fn set_alternative_accounts<I, T>(&mut self, accounts: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<IbanBic>,
    {
        self.set_field_converted(FIELD_ALTERNATIVE_ACCOUNTS, accounts, |accounts| {
            accounts
                .into_iter()
                .map(Into::into)
                .map(|acc| acc.to_string())
                .collect::<Vec<String>>()
                .join(",")
        })
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

    /// Get the payment amount as a decimal
    #[cfg(feature = "rust_decimal")]
    pub fn amount(&self) -> Result<Decimal, SpaydError> {
        self.field_converted(FIELD_AMOUNT, Decimal::from_str)
    }

    /// Set the due date from a decimal
    #[cfg(feature = "rust_decimal")]
    pub fn set_amount(&mut self, amount: &Decimal) {
        self.set_field_converted(FIELD_DUE_DATE, amount, Decimal::to_string)
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
            spayd.account(),
            Ok(IbanBic::iban_only("CZ5855000000001265098001"))
        )
    }

    #[test]
    fn acc_with_bic() {
        let spayd = Spayd::new_v1_0(vec![("ACC", "CZ5855000000001265098001+RZBCCZPP")]);
        assert_eq!(
            spayd.account(),
            Ok(IbanBic::iban_bic("CZ5855000000001265098001", "RZBCCZPP"))
        )
    }

    #[test]
    fn two_alt_accs() {
        let spayd = Spayd::new_v1_0(vec![(
            "ALT-ACC",
            "CZ5855000000001265098001+RZBCCZPP,CZ5855000000001265098001",
        )]);
        assert_eq!(
            spayd.alternative_accounts(),
            Ok(vec![
                IbanBic::iban_bic("CZ5855000000001265098001", "RZBCCZPP"),
                IbanBic::iban_only("CZ5855000000001265098001"),
            ])
        )
    }

    #[test]
    fn set_one_alt_acc() {
        let mut spayd = Spayd::empty_v1_0();
        spayd.set_alternative_accounts(vec![IbanBic::iban_bic(
            "CZ5855000000001265098001",
            "RZBCCZPP",
        )]);
        assert_eq!(
            spayd,
            Spayd::new_v1_0(vec![("ALT-ACC", "CZ5855000000001265098001+RZBCCZPP")])
        )
    }

    #[test]
    fn set_two_alt_accs() {
        let mut spayd = Spayd::empty_v1_0();
        spayd.set_alternative_accounts(vec![
            IbanBic::iban_bic("CZ5855000000001265098001", "RZBCCZPP"),
            IbanBic::iban_only("CZ5855000000001265098001"),
        ]);
        assert_eq!(
            spayd,
            Spayd::new_v1_0(vec![(
                "ALT-ACC",
                "CZ5855000000001265098001+RZBCCZPP,CZ5855000000001265098001"
            )])
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
