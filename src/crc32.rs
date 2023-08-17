#![cfg(feature = "crc32")]

use crate::spayd::Spayd;
use crc32fast::hash;

/// A success result from CRC32 checking. As the CRC32 field is optional,
/// the check can pass if no checksum was supplied. Check this value if
/// you want to enforce the usage of CRC32 checksums. Or use `require_crc32`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Crc32Ok {
    /// A CRC32 value was provided and the check passed
    Passed,
    /// The CRC32 field was not supplied
    NotProvided,
}

pub type Crc32Result = Result<Crc32Ok, ()>;

impl Crc32Ok {
    pub fn require_crc32(&self) -> Crc32Result {
        match self {
            Self::Passed => Ok(Self::Passed),
            Self::NotProvided => Err(()),
        }
    }
}

impl<'a> Spayd<'a> {
    /// Perform a CRC32 integrity check on the SPAYD to help ensure that it
    /// was received correctly. This check does not provide any assurance of
    /// the authenticity of the SPAYD value or any other form of cryptographic
    /// security.
    ///
    /// As the CRC32 field is optional, this will report success when the field
    /// is not supplied. To enforce the usage of CRC32 use require_crc32.
    pub fn check_crc32(&self) -> Crc32Result {
        if let Some(crc32_text) = self.value("CRC32") {
            // TODO: proper error
            let supplied_crc32 = u32::from_str_radix(crc32_text, 16).map_err(|_| ())?;
            let checksum = hash(self.canonic_representation().as_bytes());
            if supplied_crc32 == checksum {
                Ok(Crc32Ok::Passed)
            } else {
                Err(())
            }
        } else {
            Ok(Crc32Ok::NotProvided)
        }
    }

    /// Ensure that a CRC32 checksum is present and check that the SPAYD
    /// matches it.
    pub fn require_crc32(&self) -> Crc32Result {
        self.check_crc32()?.require_crc32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
        ]);

        assert_eq!(spayd.check_crc32(), Ok(Crc32Ok::NotProvided));
    }

    #[test]
    fn good_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
            ("CRC32", "AAD80227"),
        ]);

        assert_eq!(spayd.check_crc32(), Ok(Crc32Ok::Passed));
    }

    #[test]
    fn bad_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
            ("CRC32", "12345678"),
        ]);

        assert_eq!(spayd.check_crc32(), Err(()));
    }

    #[test]
    fn invalid_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
            ("CRC32", "JUNK"),
        ]);

        assert_eq!(spayd.check_crc32(), Err(()));
    }

    #[test]
    fn required_no_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
        ]);

        assert_eq!(spayd.require_crc32(), Err(()));
    }

    #[test]
    fn required_good_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
            ("CRC32", "AAD80227"),
        ]);

        assert_eq!(spayd.require_crc32(), Ok(Crc32Ok::Passed));
    }

    #[test]
    fn required_bad_crc32() {
        let spayd = Spayd::new_v1_0(vec![
            ("ACC", "CZ5855000000001265098001"),
            ("AM", "100.00"),
            ("CC", "CZK"),
            ("CRC32", "12345678"),
        ]);

        assert_eq!(spayd.require_crc32(), Err(()));
    }
}
