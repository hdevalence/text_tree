use super::*;
use nom::error::{context, VerboseError};
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
};

/// Use `nom`'s verbose errors for pretty-printing.
pub type IResult<I, T> = nom::IResult<I, T, VerboseError<I>>;

// == rule ===================================================================

pub(super) fn stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let (remaining, rules) = many1(skip_ws(rule))(input)?;
    let stylesheet = Stylesheet { rules };
    Ok((remaining, stylesheet))
}

fn rule(input: &str) -> IResult<&str, Rule> {
    let (remaining, (selector, declarations)) = context(
        "rule",
        pair(
            any_selector,
            delimited(
                preceded(char(' '), char('{')),
                many0(skip_ws(any_decl)),
                char('}'),
            ),
        ),
    )(input)?;

    let rule = Rule {
        selectors: vec![selector],
        declarations,
    };
    Ok((remaining, rule))
}
// == declaration ============================================================

fn any_decl(input: &str) -> IResult<&str, Declaration> {
    context(
        "any declaration",
        alt((
            named_decl("width", abs_len),
            named_decl("height", abs_len),
            // margin
            named_decl("margin", margin),
            named_decl("margin-top", margin),
            named_decl("margin-bottom", margin),
            named_decl("margin-left", margin),
            named_decl("margin-right", margin),
            // padding
            named_decl("padding", abs_len),
            named_decl("padding-top", abs_len),
            named_decl("padding-bottom", abs_len),
            named_decl("padding-left", abs_len),
            named_decl("padding-right", abs_len),
            // border
            named_decl("border", border),
            named_decl("border-top", border),
            named_decl("border-bottom", border),
            named_decl("border-left", border),
            named_decl("border-right", border),
            // display
            named_decl("display", display_kind),
            unnamed_decl,
        )),
    )(input)
}

fn named_decl<'a>(
    name: &'static str,
    value: impl FnMut(&'a str) -> IResult<&str, Value>,
) -> impl FnMut(&'a str) -> IResult<&str, Declaration> {
    context(
        name,
        map(
            delimited(
                preceded(tag(name), preceded(char(':'), char(' '))),
                value,
                char(';'),
            ),
            move |value| Declaration {
                name: String::from(name),
                value,
            },
        ),
    )
}

fn unnamed_decl<'a>(input: &str) -> IResult<&str, Declaration> {
    context(
        "declaration",
        map(
            terminated(
                pair(
                    terminated(identifier, preceded(char(':'), char(' '))),
                    take_until(";"),
                ),
                char(';'),
            ),
            |(name, value)| Declaration {
                value: Value::Keyword(String::from(value)),
                name: String::from(name),
            },
        ),
    )(input)
}

fn margin(input: &str) -> IResult<&str, Value> {
    context("margin", alt((abs_len, value(Value::Auto, tag("auto")))))(input)
}

fn display_kind(input: &str) -> IResult<&str, Value> {
    context(
        "display",
        map(
            alt((
                value(DisplayKind::Inline, tag("inline")),
                value(DisplayKind::Block, tag("block")),
                value(DisplayKind::None, tag("none")),
            )),
            Value::Display,
        ),
    )(input)
}

fn abs_len(input: &str) -> IResult<&str, Value> {
    context("absolute length", map(decimal, Value::AbsoluteLength))(input)
}

fn decimal(input: &str) -> IResult<&str, i32> {
    context(
        "decimal",
        map_res(
            recognize(preceded(
                opt(one_of("+-")),
                many1(terminated(one_of("0123456789"), many0(char('_')))),
            )),
            |s: &str| s.parse::<i32>(),
        ),
    )(input)
}

fn border(input: &str) -> IResult<&str, Value> {
    context(
        "border",
        map(
            alt((
                value(Border::None, tag("none")),
                value(Border::Light, tag("light")),
                value(Border::Heavy, tag("heavy")),
                value(Border::Double, tag("double")),
            )),
            Value::Border,
        ),
    )(input)
}

// == selector ============================================================

fn any_selector(input: &str) -> IResult<&str, Selector> {
    alt((class, id))(input)
}

fn class(input: &str) -> IResult<&str, Selector> {
    context(
        "class",
        map(preceded(char('.'), identifier), |class| Selector {
            id: None,
            classes: vec![String::from(class)],
        }),
    )(input)
}

fn id(input: &str) -> IResult<&str, Selector> {
    context(
        "id",
        map(preceded(char('#'), identifier), |id| Selector {
            id: Some(String::from(id)),
            classes: Vec::new(),
        }),
    )(input)
}

#[tracing::instrument(level = "trace", err)]
fn identifier(input: &str) -> IResult<&str, &str> {
    context(
        "identifier",
        recognize(pair(alpha1, many0(alt((alphanumeric1, tag("-")))))),
    )(input)
}

fn skip_ws<'a, T>(
    parser: impl FnMut(&'a str) -> IResult<&str, T>,
) -> impl FnMut(&'a str) -> IResult<&str, T> {
    delimited(multispace0, parser, multispace0)
}

#[cfg(test)]
mod test {
    use super::*;

    // This is primarily a demo for testng parse error formatting. Run it with
    // `cargo test -- nice_parse_errors --show-output`.
    #[test]
    fn nice_parse_errors() {
        use nom::Finish;

        let tss = r#".x { y }"#;
        let err = dbg!(stylesheet(tss)).finish().expect_err("shouldn't parse");
        println!("nice parse error: {}", nom::error::convert_error(tss, err))
    }
}
