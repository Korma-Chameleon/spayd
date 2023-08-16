use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Write};

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

pub type SpaydString<'a> = Cow<'a, str>;
type SpaydValues<'a> = BTreeMap<SpaydString<'a>, SpaydString<'a>>;

#[derive(Clone)]
pub struct Spayd<'a> {
    version: SpaydVersion,
    values: SpaydValues<'a>,
}

impl<'a> Spayd<'a> {
    pub fn new<I, K, V>(version: SpaydVersion, values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<SpaydString<'a>>,
        V: Into<SpaydString<'a>>,
    {
        Self {
            version,
            values: values
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    pub fn new_v1_0<I, K, V>(values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<SpaydString<'a>>,
        V: Into<SpaydString<'a>>,
    {
        Self::new(SpaydVersion { major: 1, minor: 0 }, values)
    }

    pub fn empty(version: SpaydVersion) -> Self {
        Self::new(version, SpaydValues::new())
    }

    pub fn empty_v1_0() -> Self {
        Self::empty(SpaydVersion { major: 1, minor: 0 })
    }

    pub fn version(&self) -> SpaydVersion {
        self.version
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(Cow::as_ref)
    }

    pub fn set_value<K, V>(&mut self, key: K, value: V)
    where
        K: Into<SpaydString<'a>>,
        V: Into<SpaydString<'a>>,
    {
        self.values.insert(key.into(), value.into());
    }

    /// Iterates over the fields in the SPAYD. No particular ordering
    /// is guaranteed.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
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

        write!(buf, "SPD*{}.{}", self.version.major, self.version.minor).unwrap();
        for (k, v) in self.iter_canonic() {
            let k = utf8_percent_encode(k, ESCAPED).to_string();
            let v = utf8_percent_encode(v, ESCAPED).to_string();
            write!(buf, "*{}:{}", k, v).unwrap();
        }
        buf
    }
}

const ESCAPED: &AsciiSet = &CONTROLS.add(b'%').add(b'*');

impl<'a> Display for Spayd<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.canonic_representation())
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
