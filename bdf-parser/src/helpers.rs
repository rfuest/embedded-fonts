use embedded_graphics::{prelude::*, primitives::Rectangle};
use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{digit1, line_ending, multispace0, one_of, space0, space1},
    combinator::{map, map_opt, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, separated_pair},
    IResult, ParseTo,
};

pub trait Parse: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

impl Parse for Point {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(separated_pair(i32::parse, space1, i32::parse), Point::from)(input)
    }
}

impl Parse for Size {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(separated_pair(u32::parse, space1, u32::parse), Size::from)(input)
    }
}

impl Parse for Rectangle {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            separated_pair(Size::parse, space1, Point::parse),
            |(size, position)| Rectangle::new(position, size),
        )(input)
    }
}

impl Parse for String {
    fn parse(input: &str) -> IResult<&str, Self> {
        map_opt(take_until_line_ending, |text: &str| text.parse_to())(input)
    }
}

impl Parse for i32 {
    fn parse(input: &str) -> IResult<&str, i32> {
        map_opt(recognize(preceded(opt(one_of("+-")), digit1)), |i: &str| {
            i.parse_to()
        })(input)
    }
}

impl Parse for u32 {
    fn parse(input: &str) -> IResult<&str, u32> {
        map_opt(recognize(digit1), |i: &str| i.parse_to())(input)
    }
}

fn comment(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("COMMENT"),
        preceded(space0, take_until_line_ending),
        line_ending,
    )(input)
}

pub fn skip_comments<'a, F, O>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(many0(comment), inner, many0(comment))
}

fn take_until_line_ending(input: &str) -> IResult<&str, &str> {
    take_while(|c| c != '\n' && c != '\r')(input)
}

pub fn statement<'a, O, F>(
    keyword: &'a str,
    parameters: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    move |input: &str| {
        let (input, _) = multispace0(input)?;
        let (input, _) = tag(keyword)(input)?;
        let (input, _) = space1(input)?;
        let (input, p) = parameters(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = opt(line_ending)(input)?;

        Ok((input, p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_takes_until_any_line_ending() {
        assert_eq!(
            take_until_line_ending("Unix line endings\n"),
            Ok(("\n".as_ref(), "Unix line endings".as_ref()))
        );

        assert_eq!(
            take_until_line_ending("Windows line endings\r\n"),
            Ok(("\r\n".as_ref(), "Windows line endings".as_ref()))
        );
    }

    #[test]
    fn it_parses_comments() {
        let comment_text = "COMMENT test text\n";
        let out = comment(comment_text);

        assert_eq!(out, Ok(("", "test text")));

        // EMPTY comments
        assert_eq!(comment("COMMENT\n"), Ok(("", "")));
    }
}
