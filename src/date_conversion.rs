use crate::Spayd;
use chrono::NaiveDate;
use thiserror::Error;

const SPAYD_DATE_FMT: &str = "%Y%m%d";
const DUE_DATE: &str = "DT";

/// Errors encountered when handling date fields.
#[derive(Error, Debug, PartialEq)]
pub enum DateError {
    /// Parsing failed. The date has an incorrect format.
    #[error("couldn't parse date '{0}'")]
    ParseError(String),
    /// Thd SPAYD value doesn't have the date field.
    #[error("field '{0}' is missing")]
    FieldMissing(String),
}

impl<'a> Spayd<'a> {
    /// Get the due date as a Chrono NaiveDate
    // TODO: error type
    pub fn due_date(&self) -> Result<NaiveDate, DateError> {
        if let Some(date_text) = self.field(DUE_DATE) {
            NaiveDate::parse_from_str(date_text, SPAYD_DATE_FMT)
                .or(Err(DateError::ParseError(date_text.into())))
        } else {
            Err(DateError::FieldMissing(DUE_DATE.into()))
        }
    }

    /// Set the due date from a Chrono NaiveDate
    pub fn set_due_date(&mut self, date: &NaiveDate) {
        let text = date.format(SPAYD_DATE_FMT).to_string();
        self.set_field(DUE_DATE, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::format::ParseErrorKind;

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
            Err(DateError::ParseError("2012/12/31".into()))
        )
    }

    #[test]
    fn set_due_date_correct_format() {
        let mut spayd = Spayd::empty_v1_0();
        spayd.set_due_date(&NaiveDate::from_ymd_opt(2012, 12, 31).unwrap());
        assert_eq!(spayd.field("DT"), Some("20121231"))
    }
}
