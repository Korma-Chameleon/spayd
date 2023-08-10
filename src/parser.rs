use crate::spayd::SpaydVersion;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::digit1,
    combinator::{map, map_res},
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

fn version_section(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn version(input: &str) -> IResult<&str, SpaydVersion> {
    map(
        separated_pair(version_section, tag("."), version_section),
        |(major, minor)| SpaydVersion::new(major, minor),
    )(input)
}

fn header(input: &str) -> IResult<&str, SpaydVersion> {
    delimited(tag("SPD*"), version, tag("*"))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version() {
        assert_eq!(version("0.1"), Ok(("", SpaydVersion::new(0, 1))));
        assert_eq!(version("1.0"), Ok(("", SpaydVersion::new(1, 0))));
        assert_eq!(version("1.5"), Ok(("", SpaydVersion::new(1, 5))));
        assert_eq!(version("2.1"), Ok(("", SpaydVersion::new(2, 1))));
    }

    #[test]
    fn parse_heaser() {
        assert_eq!(header("SPD*1.0*"), Ok(("", SpaydVersion::new(1, 0))));
        assert_eq!(
            header("SPD*1.0*ACC:..."),
            Ok(("ACC:...", SpaydVersion::new(1, 0)))
        );
    }
}
