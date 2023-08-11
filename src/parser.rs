use crate::spayd::{Spayd, SpaydValues, SpaydVersion};
use nom::{
    bytes::complete::{is_not, tag, take_until1, take_while},
    character::complete::digit1,
    combinator::{all_consuming, map, map_parser, map_res},
    error::Error,
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    Finish, IResult,
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
    map(separated_list1(tag("*"), kv_pair), |items| {
        items
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    })(input)
}

fn full_text(input: &str) -> IResult<&str, Spayd> {
    map(pair(header, values), |(version, values)| {
        Spayd::new(version, values)
    })(input)
}

fn is_ascii_printable(c: char) -> bool {
    c.is_ascii() && !c.is_ascii_control()
}

pub fn parse_spayd(input: &str) -> Result<Spayd, Error<&str>> {
    let parsed =
        all_consuming(map_parser(take_while(is_ascii_printable), full_text))(input).finish()?;
    Ok(parsed.1)
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

    #[test]
    fn full_example() {
        let spayd = parse_spayd(
            "SPD*1.0*ACC:CZ5855000000001265098001*AM:480.50*CC:CZK*MSG:Payment for the goods",
        )
        .unwrap();

        assert_eq!(spayd.version(), SpaydVersion::new(1, 0));
        assert_eq!(spayd.value("ACC"), Some("CZ5855000000001265098001"));
        assert_eq!(spayd.value("AM"), Some("480.50"));
        assert_eq!(spayd.value("CC"), Some("CZK"));
        assert_eq!(spayd.value("MSG"), Some("Payment for the goods"));
        assert_eq!(spayd.value("ALT-ACC"), None);
        assert_eq!(spayd.value("RF"), None);
    }

    #[test]
    fn incomplete() {
        assert!(parse_spayd("SPD*1.0").is_err());
        assert!(parse_spayd("SPD*1.0*").is_err());
        assert!(parse_spayd("SPD*1.0*ACC").is_err());
        assert!(parse_spayd("SPD*1.0*ACC:").is_err());
    }

    #[test]
    fn non_ascii() {
        assert!(parse_spayd("SPD*1.0*PŘÍKLAD:123").is_err());
    }
}
