use crate::Spayd;
use chrono::NaiveDate;

const SPAYD_DATE_FMT: &str = "%Y%m%d";
const DUE_DATE: &str = "DT";

impl<'a> Spayd<'a> {
    /// Get the due date as a Chrono NaiveDate
    // TODO: error type
    pub fn due_date(&self) -> Option<NaiveDate> {
        self.field(DUE_DATE)
            .and_then(|date_text| NaiveDate::parse_from_str(date_text, SPAYD_DATE_FMT).ok())
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

    #[test]
    fn due_date_correct_format() {
        let spayd = Spayd::new_v1_0(vec![("DT", "20121231")]);
        assert_eq!(spayd.due_date(), NaiveDate::from_ymd_opt(2012, 12, 31))
    }

    #[test]
    fn due_date_incorrect_format() {
        let spayd = Spayd::new_v1_0(vec![("DT", "2012/12/31")]);
        assert_eq!(spayd.due_date(), None)
    }

    #[test]
    fn set_due_date_correct_format() {
        let mut spayd = Spayd::empty_v1_0();
        spayd.set_due_date(&NaiveDate::from_ymd_opt(2012, 12, 31).unwrap());
        assert_eq!(spayd.field("DT"), Some("20121231"))
    }
}
