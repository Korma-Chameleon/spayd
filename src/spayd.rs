use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use crate::SpaydError;

/// Version number of the Short Payment Descriptor.
///
/// Currently there is only a standard for version 1.0.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SpaydVersion {
    pub major: u32,
    pub minor: u32,
}

impl SpaydVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl Display for SpaydVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SPD*{}.{}", self.major, self.minor)
    }
}

type SpaydFields = BTreeMap<String, String>;

/// A Short Payment Descriptor structure containint the details of
/// a requested payment.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spayd {
    version: SpaydVersion,
    fields: SpaydFields,
}

impl<'a> Spayd {
    /// Create a new SPAYD with the given version number and field values.
    /// Using `new_v1_0` or `empty_v1_0` is preferable for most situations.
    pub fn new<I, K, V>(version: SpaydVersion, fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        Self {
            version,
            fields: fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    /// Create a version 1.0 SPAYD with the given field values.
    pub fn new_v1_0<I, K, V>(fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        Self::new(SpaydVersion { major: 1, minor: 0 }, fields)
    }

    /// Create an empty version 1.0 SPAYD.
    pub fn empty_v1_0() -> Self {
        Self::new_v1_0(SpaydFields::new())
    }

    /// Get the version number.
    pub fn version(&self) -> SpaydVersion {
        self.version
    }

    /// Get the value of the given field.
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_ref)
    }

    /// Set the value of the given field.
    pub fn set_field<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: ToString,
    {
        self.fields.insert(key.to_string(), value.to_string());
    }

    /// Ensure that all required fields are present. In version 1.0 this
    /// is only the ACC field.
    fn check_required_fields(&self) -> Result<(), SpaydError> {
        if self.field("ACC").is_none() {
            Err(SpaydError::RequiredFieldMissing("ACC".into()))
        } else {
            Ok(())
        }
    }

    /// Ensure that the SPAYD has all required fields and that the CRC
    /// check is correct (if this feature is enabled).
    pub fn validate(&self) -> Result<(), SpaydError> {
        self.check_required_fields()?;

        #[cfg(feature = "crc32")]
        self.check_crc32()?;

        Ok(())
    }

    /// Iterates over the fields in the SPAYD. No particular ordering
    /// is guaranteed.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
    }

    /// Iterates over the fields in canonic order. The keys are
    /// alphabetical and the CRC32 field is excluded. This can be used to
    /// create a cannonical represenataion of the SPAYD which can be CRC32 checked.
    pub fn iter_canonic(&self) -> impl Iterator<Item = (&str, &str)> {
        // As the fields are stored in a BTreeMap, they will have the right
        // order. This will need to be updated if the storage is changed.
        self.iter().filter(|(k, _)| *k != "CRC32")
    }

    /// Construct canonic representation for CRC32 checking
    pub fn canonic_representation(&self) -> String {
        let mut buf = String::new();

        buf.push_str(&self.version.to_string());
        buf.push_str(&Self::fields_to_string(&mut self.iter_canonic()));

        buf
    }

    /// Format fields into a string according to the SPAYD standard.
    fn fields_to_string(fields: &mut dyn Iterator<Item = (&str, &str)>) -> String {
        let mut buf = String::new();

        for (k, v) in fields {
            buf.push('*');
            buf.push_str(&utf8_percent_encode(k, ESCAPED).to_string());

            buf.push(':');
            buf.push_str(&utf8_percent_encode(v, ESCAPED).to_string());
        }
        buf
    }
}

const ESCAPED: &AsciiSet = &CONTROLS.add(b'%').add(b'*');

impl Display for Spayd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.version,
            Self::fields_to_string(&mut self.iter())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_1_0() {
        let spayd = Spayd::empty_v1_0();
        assert_eq!(spayd.version().major, 1);
        assert_eq!(spayd.version().minor, 0);

        assert_eq!(spayd.to_string(), "SPD*1.0");
    }

    #[test]
    fn iter_canonical() {
        let spayd = Spayd::new_v1_0(vec![
            ("CC", "CZK"),
            ("MSG", "Payment for the goods"),
            ("AM", "480.50"),
            ("ACC", "CZ5855000000001265098001"),
            ("CRC32", "JUNKDATA"),
        ]);
        let mut iterator = spayd.iter_canonic();

        assert_eq!(iterator.next(), Some(("ACC", "CZ5855000000001265098001")));
        assert_eq!(iterator.next(), Some(("AM", "480.50")));
        assert_eq!(iterator.next(), Some(("CC", "CZK")));
        assert_eq!(iterator.next(), Some(("MSG", "Payment for the goods")));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn cannonical_string() {
        let spayd = Spayd::new_v1_0(vec![
            ("CC", "CZK"),
            ("MSG", "Payment for the goods"),
            ("AM", "480.50"),
            ("ACC", "CZ5855000000001265098001"),
            ("CRC32", "JUNKDATA"),
        ]);

        assert_eq!(
            spayd.canonic_representation(),
            "SPD*1.0*ACC:CZ5855000000001265098001*AM:480.50*CC:CZK*MSG:Payment for the goods"
        );
    }

    #[test]
    fn to_string() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "480.50"),
            ("CC", "CZK"),
            ("MSG", "Payment for the goods"),
        ]);

        assert_eq!(
            spayd.to_string(),
            "SPD*1.0*ACC:CZ5855000000001265098001*AM:480.50*CC:CZK*MSG:Payment for the goods"
        );
    }

    #[test]
    fn percent_encoding() {
        let spayd = Spayd::new_v1_0(vec![("MSG", "****!")]);
        assert_eq!(spayd.to_string(), "SPD*1.0*MSG:%2A%2A%2A%2A!");

        let spayd = Spayd::new_v1_0(vec![("MSG", "PŘÍKLAD")]);
        assert_eq!(spayd.to_string(), "SPD*1.0*MSG:P%C5%98%C3%8DKLAD");
    }
}
