# Changelog

## [0.2.0] - 2023-08-17

### Added

- Format conversions for commonly used data types, decimal for currency, chrono for dates etc
- Support for FromStr and str::parse()

### Changed

- Spayd data structures now use owned Strings internally and don't require a lifetime parameter


## [0.1.2] - 2023-08-17

### Added

- Data structures for Spayd values, including version and data fields
- Baisc parser for conversion from a string
- Conversion of Spayd values into strings
- Canonic representation and CRC32 checking
