use crate::spayd::{Spayd, SpaydString, SpaydVersion};
use nom::{
    bytes::complete::{is_not, tag, take_until1, take_while},
    character::complete::digit1,
    combinator::{all_consuming, map, map_parser, map_res},
    error::{Error, ErrorKind},
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    Finish, IResult,
};
use percent_encoding::percent_decode_str;

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

fn decode_percent_encoding(i: &str) -> Result<SpaydString, Error<&str>> {
    match percent_decode_str(i).decode_utf8() {
        Ok(t) => Ok(t.into()),
        Err(_) => Err(Error::new(i, ErrorKind::Escaped)),
    }
}

fn decode_spayd_kv<'a>(
    k: &'a str,
    v: &'a str,
) -> Result<(SpaydString<'a>, SpaydString<'a>), Error<&'a str>> {
    let k = decode_percent_encoding(k)?;
    let v = decode_percent_encoding(v)?;
    Ok((k, v))
}

fn kv_pair(input: &str) -> IResult<&str, (SpaydString, SpaydString)> {
    map_res(
        separated_pair(take_until1(":"), tag(":"), is_not("*")),
        |(k, v)| decode_spayd_kv(k, v),
    )(input)
}

fn values(input: &str) -> IResult<&str, Vec<(SpaydString, SpaydString)>> {
    separated_list1(tag("*"), kv_pair)(input)
}

fn full_text(input: &str) -> IResult<&str, Spayd> {
    map(pair(header, values), |(version, values)| {
        Spayd::new(version, values)
    })(input)
}

fn is_ascii_printable(c: char) -> bool {
    c.is_ascii() && !c.is_ascii_control()
}

/// Parse text into a Spayd value.
pub fn parse_spayd(input: &str) -> Result<Spayd, Error<&str>> {
    let parsed =
        all_consuming(map_parser(take_while(is_ascii_printable), full_text))(input).finish()?;
    Ok(parsed.1)
}

#[cfg(test)]
mod tests {
    // Most xxample data is from wikipedia
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
            Ok(("", ("ACC".into(), "CZ5855000000001265098001".into())))
        );
        assert_eq!(
            kv_pair("AM:480.50"),
            Ok(("", ("AM".into(), "480.50".into())))
        );
        assert_eq!(
            kv_pair("MSG:Payment for the goods"),
            Ok(("", ("MSG".into(), "Payment for the goods".into())))
        );
    }

    #[test]
    fn percent_encoded_kv() {
        assert_eq!(
            kv_pair("MSG:%40%3F%2A%24%21"),
            Ok(("", ("MSG".into(), "@?*$!".into())))
        );
        assert_eq!(
            kv_pair("RN:Krte%C4%8Dek"),
            Ok(("", ("RN".into(), "Krteček".into())))
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
            kv_pairs,
            vec![
                ("ACC".into(), "CZ5855000000001265098001".into()),
                ("AM".into(), "480.50".into()),
                ("CC".into(), "CZK".into()),
                ("MSG".into(), "Payment for the goods".into())
            ]
        );
    }

    #[test]
    fn percent_encoded_values() {
        let parsed = values("MSG:%40%3F%2A%24%21").unwrap();
        let kv_pairs = parsed.1;

        assert_eq!(kv_pairs, vec![("MSG".into(), "@?*$!".into())]);
    }

    #[test]
    fn full_example() {
        let spayd = parse_spayd(
            "SPD*1.0*ACC:CZ5855000000001265098001*AM:480.50*CC:CZK*MSG:Payment for the goods",
        )
        .unwrap();

        assert_eq!(spayd.version(), SpaydVersion::new(1, 0));
        assert_eq!(spayd.field("ACC"), Some("CZ5855000000001265098001"));
        assert_eq!(spayd.field("AM"), Some("480.50"));
        assert_eq!(spayd.field("CC"), Some("CZK"));
        assert_eq!(spayd.field("MSG"), Some("Payment for the goods"));
        assert_eq!(spayd.field("ALT-ACC"), None);
        assert_eq!(spayd.field("RF"), None);
    }

    #[test]
    fn percent_encoded() {
        let spayd = parse_spayd("SPD*1.0*MSG:%40%3F%2A%24%21").unwrap();

        assert_eq!(spayd.field("MSG"), Some("@?*$!"));
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
