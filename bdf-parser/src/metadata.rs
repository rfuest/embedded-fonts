use embedded_graphics::{prelude::*, primitives::Rectangle};
use nom::{
    character::complete::{multispace0, space1},
    combinator::{map, map_res},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub version: f32,
    pub name: String,
    pub point_size: i32,
    pub resolution: (u32, u32),
    pub bounding_box: Rectangle,
}
fn metadata_version(input: &[u8]) -> IResult<&[u8], f32> {
    map_res(statement("STARTFONT", String::parse), |text| text.parse())(input)
}

fn metadata_name(input: &[u8]) -> IResult<&[u8], String> {
    statement("FONT", String::parse)(input)
}

fn metadata_size(input: &[u8]) -> IResult<&[u8], (i32, (u32, u32))> {
    statement(
        "SIZE",
        separated_pair(
            i32::parse,
            space1,
            map(Size::parse, |size| (size.width, size.height)),
        ),
    )(input)
}

fn metadata_bounding_box(input: &[u8]) -> IResult<&[u8], Rectangle> {
    statement("FONTBOUNDINGBOX", Rectangle::parse)(input)
}

pub fn header(input: &[u8]) -> IResult<&[u8], Metadata> {
    let (input, version) = preceded(optional_comments, metadata_version)(input)?;
    let (input, name) = preceded(optional_comments, metadata_name)(input)?;
    let (input, (point_size, resolution)) = preceded(optional_comments, metadata_size)(input)?;
    let (input, bounding_box) = preceded(optional_comments, metadata_bounding_box)(input)?;
    let (input, _) = optional_comments(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Metadata {
            version,
            name,
            point_size,
            resolution,
            bounding_box,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::geometry::{Point, Size};

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_the_font_version() {
        assert_eq!(metadata_version(b"STARTFONT 2.1\n"), Ok((EMPTY, 2.1f32)));

        // Some fonts are a bit overzealous with their whitespace
        assert_eq!(metadata_version(b"STARTFONT   2.1\n"), Ok((EMPTY, 2.1f32)));
    }

    #[test]
    fn it_parses_header() {
        let input = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 75
FONTBOUNDINGBOX 16 24 0 0"#;

        assert_eq!(
            header(input.as_bytes()),
            Ok((
                EMPTY,
                Metadata {
                    version: 2.1,
                    name: String::from("\"test font\""),
                    point_size: 16,
                    resolution: (75, 75),
                    bounding_box: Rectangle::new(Point::zero(), Size::new(16, 24))
                }
            ))
        );
    }
}
