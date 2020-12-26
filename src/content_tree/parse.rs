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

/// The contents of an HTML tag, *without* any delimiters.
///
/// This is factored out so that the same code can be used for `<foo>...</foo>`
/// and for `<foo />`.
#[tracing::instrument(level = "trace", err)]
fn attrs(input: &str) -> IResult<&str, ElementData> {
    let (remaining, attrs) = opt(preceded(
        multispace1,
        separated_list0(multispace1, any_attribute),
    ))(input)?;

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
    Ok((remaining, ElementData { id, classes }))
}

#[tracing::instrument(level = "trace", err)]
fn open_tag(input: &str) -> IResult<&str, ElementData> {
    let (remaining, (tag_name, mut attrs)) =
        delimited(tag("<"), pair(identifier, attrs), tag(">"))(input)?;
    attrs.classes.insert(tag_name.to_string());
    Ok((remaining, attrs))
}

fn close_tag<'a>(name: &'a str) -> impl FnMut(&'a str) -> IResult<&str, &str> {
    move |input| {
        let span = tracing::trace_span!("close_tag", ?name, ?input,);
        let _e = span.enter();
        recognize(delimited(tag("</"), tag_no_case(name), tag(">")))(input)
    }
}

#[tracing::instrument(level = "trace", err)]
fn tag_no_children(input: &str) -> IResult<&str, Node> {
    let (remaining, (tag_name, mut attrs)) = delimited(
        tag("<"),
        pair(identifier, attrs),
        pair(multispace1, tag("/>")),
    )(input)?;
    attrs.classes.insert(tag_name.to_string());
    Ok((
        remaining,
        Node {
            children: Vec::new(),
            node_data: NodeData::Element(attrs),
        },
    ))
}

#[tracing::instrument(level = "trace", err)]
fn tag_children(input: &str) -> IResult<&str, Node> {
    let (remaining, (tag_name, mut attrs)) = delimited(
        tag("<"),
        pair(identifier, attrs),
        pair(multispace0, tag(">")),
    )(input)?;
    attrs.classes.insert(tag_name.to_string());
    let (remaining, children) = terminated(many0(element), close_tag(tag_name))(remaining)?;
    Ok((
        remaining,
        Node {
            children,
            node_data: NodeData::Element(attrs),
        },
    ))
}

#[tracing::instrument(level = "trace", err)]
fn element(input: &str) -> IResult<&str, Node> {
    alt((tag_no_children, tag_children))(input)
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

    #[test]
    fn simple_nested() {
        trace_init();

        let html = "<a><b></b></a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        let b = Node {
            children: Vec::new(),
            node_data: NodeData::Element(ElementData {
                classes: Some("b".to_string()).into_iter().collect(),
                id: None,
            }),
        };
        let a = Node {
            children: vec![b],
            node_data: NodeData::Element(ElementData {
                classes: Some("a".to_string()).into_iter().collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn bad_nesting_doesnt_parse() {
        trace_init();

        let html = "<a></b><b></a>";
        let res = dbg!(element(html));
        assert!(res.is_err(), "html: {:?}", html);

        let html = "<a>";
        let res = dbg!(element(html));
        assert!(res.is_err(), "html: {:?}", html);

        let html = "<a><b>";
        let res = dbg!(element(html));
        assert!(res.is_err(), "html: {:?}", html);

        let html = "<a></b></a>";
        let res = dbg!(element(html));
        assert!(res.is_err(), "html: {:?}", html);

        let html = "<a><b></a>";
        let res = dbg!(element(html));
        assert!(res.is_err(), "html: {:?}", html);
    }
}
