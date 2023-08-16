use std::borrow::Cow;
use std::collections::BTreeMap;

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
    pub fn new<V>(version: SpaydVersion, values: V) -> Self
    where
        V: IntoIterator<Item = (SpaydString<'a>, SpaydString<'a>)>,
    {
        Self {
            version,
            values: values.into_iter().collect(),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_1_0() {
        let spayd = Spayd::empty_v1_0();
        assert_eq!(spayd.version().major, 1);
        assert_eq!(spayd.version().minor, 0);
    }
}
