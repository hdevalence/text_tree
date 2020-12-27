use super::*;
use nom::error::{context, VerboseError};
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
};

/// Use `nom`'s verbose errors for pretty-printing.
pub type IResult<I, T> = nom::IResult<I, T, VerboseError<I>>;

#[tracing::instrument(level = "trace", err)]
pub(super) fn document(input: &str) -> IResult<&str, Node> {
    preceded(
        opt(delimited(
            tag_no_case("<!doctype"),
            take_until(">"),
            tag(">"),
        )),
        skip_ws(context("root tag", tag_children)),
    )(input)
}

#[tracing::instrument(level = "trace", err)]
pub(super) fn element(input: &str) -> IResult<&str, Node> {
    alt((
        context("tag with children", tag_children),
        context("tag without children", tag_no_children),
        text,
    ))(input)
}

#[tracing::instrument(level = "trace", err)]
fn text(input: &str) -> IResult<&str, Node> {
    let (remaining, text) = context("text", preceded(not(tag("<")), take_until("<")))(input)?;
    let re = regex::Regex::new("\\s+").unwrap();
    let text = re.replace_all(text, " ");
    Ok((remaining, Node::from(text)))
}

#[tracing::instrument(level = "trace", err)]
fn identifier(input: &str) -> IResult<&str, &str> {
    context(
        "identifier",
        recognize(pair(
            alt((alpha1, tag("_"), tag(":"))),
            many0(alt((alphanumeric1, tag("_"), tag(":"), tag(".")))),
        )),
    )(input)
}

#[tracing::instrument(level = "trace", err)]
fn attribute_value(input: &str) -> IResult<&str, &str> {
    context(
        "attribute value",
        preceded(tag("="), delimited(tag("\""), take_until("\""), tag("\""))),
    )(input)
}

#[tracing::instrument(level = "trace", err)]
fn any_attribute(input: &str) -> IResult<&str, (&str, &str)> {
    context("attribute", pair(identifier, attribute_value))(input)
}

/// The contents of an HTML tag, *without* any delimiters.
///
/// This is factored out so that the same code can be used for `<foo>...</foo>`
/// and for `<foo />`.
#[tracing::instrument(level = "trace", err)]
fn attrs(input: &str) -> IResult<&str, ElementData> {
    let (remaining, attrs) = opt(preceded(ws1, separated_list0(ws1, any_attribute)))(input)?;

    let mut classes = HashSet::new();
    let mut id = None;
    for (name, val) in attrs.into_iter().flat_map(IntoIterator::into_iter) {
        match name {
            n if n.eq_ignore_ascii_case("class") => {
                classes = val.split(' ').map(String::from).collect();
            }
            n if n.eq_ignore_ascii_case("id") => {
                id = Some(val.to_string());
            }
            n if n.eq_ignore_ascii_case("style") => {
                todo!("style attributes should probably 'work'...")
            }
            _ => {
                // Skip other attributes for now...
            }
        }
    }
    tracing::debug!(?classes, ?id, "parsed attrs");
    Ok((remaining, ElementData { id, classes }))
}

#[tracing::instrument(level = "trace", err)]
#[cfg(test)]
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

fn ws1(input: &str) -> IResult<&str, ()> {
    value((), many1(alt((comment, multispace1))))(input)
}

fn non_space_ws1(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many1(alt((
            comment,
            preceded(tag("\r\n"), multispace0),
            preceded(tag("\n"), multispace0),
            tag("\t"),
        ))),
    )(input)
}

fn skip_non_space_ws<'a, T>(
    parser: impl FnMut(&str) -> IResult<&str, T>,
) -> impl FnMut(&'a str) -> IResult<&str, T> {
    delimited(opt(non_space_ws1), parser, opt(non_space_ws1))
}

fn skip_ws<'a, T>(
    parser: impl FnMut(&'a str) -> IResult<&str, T>,
) -> impl FnMut(&'a str) -> IResult<&str, T> {
    delimited(opt(ws1), parser, opt(ws1))
}

#[tracing::instrument(level = "trace", err)]
fn comment(input: &str) -> IResult<&str, &str> {
    delimited(tag("<!--"), take_until("-->"), tag("-->"))(input)
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
    let (remaining, (tag_name, mut attrs)) = context(
        "open tag",
        delimited(
            tag("<"),
            pair(identifier, attrs),
            preceded(multispace0, tag(">")),
        ),
    )(input)?;
    attrs.classes.insert(tag_name.to_string());
    let (remaining, children) = terminated(
        // Only skip "non-space" whitespace here. Spaces may be part of a
        // text node.
        context("children", many0(skip_non_space_ws(element))),
        context("close tag", close_tag(tag_name)),
    )(remaining)?;
    Ok((
        remaining,
        Node {
            children,
            node_data: NodeData::Element(attrs),
        },
    ))
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

    #[test]
    fn newline() {
        trace_init();

        let html = "<a>\n</a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
        let a = Node {
            children: vec![],
            node_data: NodeData::Element(ElementData {
                classes: Some("a".to_string()).into_iter().collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn newline_text() {
        trace_init();

        let html = "<a>\nHello world\n</a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
        let a = Node {
            children: vec![Node::from("Hello world\n")],
            node_data: NodeData::Element(ElementData {
                classes: Some("a".to_string()).into_iter().collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn text() {
        trace_init();

        let html = "Hello world!";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");

        assert_eq!(parsed, Node::from("Hello world!"));
        assert_eq!(remaining, "");
    }

    #[test]
    fn simple_nested_text_1() {
        trace_init();

        let html = "<a>Hello world!</a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
        let a = Node {
            children: vec![Node::from("Hello world!")],
            node_data: NodeData::Element(ElementData {
                classes: Some("a".to_string()).into_iter().collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn simple_nested_text_2() {
        trace_init();

        let html = "<a>Hello <b>world</b>!</a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
        let b = Node {
            children: vec![Node::from("world")],
            node_data: NodeData::Element(ElementData {
                classes: Some("b".to_string()).into_iter().collect(),
                id: None,
            }),
        };
        let a = Node {
            children: vec![Node::from("Hello "), b, Node::from("!")],
            node_data: NodeData::Element(ElementData {
                classes: Some("a".to_string()).into_iter().collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn simple_nested_text_attrs() {
        trace_init();

        let html =
            "<a class=\"foo bar\" href=\"my cool website\">Hello <b id=\"thing\">world</b>!</a>";
        let (remaining, parsed) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
        let b = Node {
            children: vec![Node::from("world")],
            node_data: NodeData::Element(ElementData {
                classes: Some("b".to_string()).into_iter().collect(),
                id: Some("thing".to_string()),
            }),
        };
        let a = Node {
            children: vec![Node::from("Hello "), b, Node::from("!")],
            node_data: NodeData::Element(ElementData {
                classes: vec!["a", "foo", "bar"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                id: None,
            }),
        };

        assert_eq!(parsed, a);
        assert_eq!(remaining, "");
    }

    #[test]
    fn bad_p() {
        trace_init();

        let html = r#"<p>This domain is for use in illustrative examples in documents. You may use this
        domain in literature without prior coordination or asking for permission.</p>"#;
        let (_, _) = dbg!(element(html))
            .map_err(|e| e.to_string())
            .expect("it should parse");
    }
}
