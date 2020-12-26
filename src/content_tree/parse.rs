use super::*;
use nom::error::VerboseError;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
};

/// Use `nom`'s verbose errors for pretty-printing.
pub type IResult<I, T> = nom::IResult<I, T, VerboseError<I>>;

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"), tag(":"))),
        many0(alt((alphanumeric1, tag("_"), tag(":"), tag(".")))),
    ))(input)
}

fn open_tag(input: &str) -> IResult<&str, ElementData> {
    let mut parse_open_tag = delimited(tag("<"), identifier, tag(">"));
    let (remaining, name) = parse_open_tag(input)?;
    let mut classes = HashSet::new();
    classes.insert(String::from(name));
    Ok((
        remaining,
        ElementData {
            // TODO(eliza)
            id: None,
            classes,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_open_tag() {
        let open = "<a>";
        let (remaining, parsed) = dbg!(open_tag(open)).expect("it should parse");

        let mut classes = HashSet::new();
        classes.insert(String::from("a"));
        assert_eq!(parsed, ElementData { classes, id: None });
        assert_eq!(remaining, "");
    }
}
