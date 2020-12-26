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

fn attribute_value(input: &str) -> IResult<&str, &str> {
    preceded(tag("="), delimited(tag("\""), take_until("\""), tag("\"")))(input)
}

fn named_attribute<'a>(name: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    preceded(tag(name), attribute_value)
}

fn any_attribute(input: &str) -> IResult<&str, (&str, &str)> {
    pair(identifier, attribute_value)(input)
}

fn open_tag(input: &str) -> IResult<&str, ElementData> {
    let (remaining, (tag_name, attrs)) = delimited(
        tag("<"),
        pair(
            identifier,
            opt(preceded(
                multispace1,
                separated_list0(multispace1, any_attribute),
            )),
        ),
        tag(">"),
    )(input)?;

    let mut classes = HashSet::new();
    let mut id = None;
    for (name, val) in attrs.into_iter().flat_map(IntoIterator::into_iter) {
        match name {
            "class" => {
                classes = val.split(' ').map(String::from).collect();
            }
            "id" => {
                id = Some(val.to_string());
            }
            "style" => todo!("style attributes should probably 'work'..."),
            _ => {
                // Skip other attributes for now...
            }
        }
    }
    classes.insert(tag_name.to_string());
    Ok((remaining, ElementData { id, classes }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_open_tag() {
        let open = "<a>";
        let (remaining, parsed) = dbg!(open_tag(open))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let mut classes = HashSet::new();
        classes.insert(String::from("a"));
        assert_eq!(parsed, ElementData { classes, id: None });
        assert_eq!(remaining, "");
    }

    #[test]
    fn open_tag_with_ignored_attrs() {
        let open = "<a href=\"my cool website\">";
        let (remaining, parsed) = dbg!(open_tag(open))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let mut classes = HashSet::new();
        classes.insert(String::from("a"));
        assert_eq!(parsed, ElementData { classes, id: None });
        assert_eq!(remaining, "");
    }

    #[test]
    fn open_tag_with_classes() {
        let open = "<a class=\"foo bar baz\">";
        let (remaining, parsed) = dbg!(open_tag(open))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let classes = vec!["a", "foo", "bar", "baz"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(parsed, ElementData { classes, id: None });
        assert_eq!(remaining, "");
    }

    #[test]
    fn open_tag_with_classes_and_attrs() {
        let open = "<a href=\"my website\" class=\"foo bar baz\" something=\"lol\">";
        let (remaining, parsed) = dbg!(open_tag(open))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let classes = vec!["a", "foo", "bar", "baz"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(parsed, ElementData { classes, id: None });
        assert_eq!(remaining, "");
    }

    #[test]
    fn open_tag_with_classes_and_id() {
        let open = "<a href=\"my website\" class=\"foo bar baz\" id=\"cool\">";
        let (remaining, parsed) = dbg!(open_tag(open))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let classes = vec!["a", "foo", "bar", "baz"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(
            parsed,
            ElementData {
                classes,
                id: Some("cool".to_string())
            }
        );
        assert_eq!(remaining, "");
    }
}
