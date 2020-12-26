use super::*;
use nom::error::VerboseError;
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
};

/// Use `nom`'s verbose errors for pretty-printing.
pub type IResult<I, T> = nom::IResult<I, T, VerboseError<I>>;

#[tracing::instrument(level = "trace", err)]
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"), tag(":"))),
        many0(alt((alphanumeric1, tag("_"), tag(":"), tag(".")))),
    ))(input)
}

#[tracing::instrument(level = "trace", err)]
fn attribute_value(input: &str) -> IResult<&str, &str> {
    preceded(tag("="), delimited(tag("\""), take_until("\""), tag("\"")))(input)
}

#[tracing::instrument(level = "trace", err)]
fn any_attribute(input: &str) -> IResult<&str, (&str, &str)> {
    pair(identifier, attribute_value)(input)
}

#[tracing::instrument(level = "trace", err)]
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

    fn trace_init() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_max_level(tracing::Level::TRACE)
            .try_init();
    }

    #[test]
    fn simple_open_tag() {
        trace_init();

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
        trace_init();

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
        trace_init();

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
        trace_init();

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
        trace_init();

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
