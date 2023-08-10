use crate::spayd::{SpaydValues, SpaydVersion};
use nom::{
    bytes::complete::{is_not, tag, take_until1},
    character::complete::digit1,
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::{delimited, separated_pair, terminated},
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

fn kv_pair(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(take_until1(":"), tag(":"), is_not("*"))(input)
}

fn values(input: &str) -> IResult<&str, SpaydValues> {
    map(separated_list0(tag("*"), kv_pair), |items| {
        items
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    })(input)
}

#[cfg(test)]
mod tests {
    // All xxample data is from wikipedia
    // https://en.wikipedia.org/wiki/Short_Payment_Descriptor
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

    #[test]
    fn parse_kv() {
        assert_eq!(
            kv_pair("ACC:CZ5855000000001265098001"),
            Ok(("", ("ACC", "CZ5855000000001265098001")))
        );
        assert_eq!(kv_pair("AM:480.50"), Ok(("", ("AM", "480.50"))));
        assert_eq!(
            kv_pair("MSG:Payment for the goods"),
            Ok(("", ("MSG", "Payment for the goods")))
        );
    }

    #[test]
    fn parse_values() {
        let parsed =
            values("ACC:CZ5855000000001265098001*AM:480.50*CC:CZK*MSG:Payment for the goods")
                .unwrap();
        assert_eq!(parsed.0, "");

        let kv_pairs = parsed.1;
        assert_eq!(
            kv_pairs.get("ACC").map(AsRef::as_ref),
            Some("CZ5855000000001265098001")
        );
        assert_eq!(kv_pairs.get("AM").map(AsRef::as_ref), Some("480.50"));
        assert_eq!(kv_pairs.get("CC").map(AsRef::as_ref), Some("CZK"));
        assert_eq!(
            kv_pairs.get("MSG").map(AsRef::as_ref),
            Some("Payment for the goods")
        );
    }
}
