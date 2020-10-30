use embedded_graphics::prelude::*;
use nom::{
    character::complete::{multispace0, space1},
    combinator::{map, map_res},
    sequence::separated_pair,
    IResult,
};

use crate::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub version: f32,
    pub name: String,
    pub point_size: i32,
    pub resolution: (u32, u32),
    pub bounding_box: BoundingBox,
}

impl Parse for Metadata {
    fn parse(input: &str) -> IResult<&str, Metadata> {
        let (input, version) = skip_comments(metadata_version)(input)?;
        let (input, name) = skip_comments(metadata_name)(input)?;
        let (input, (point_size, resolution)) = skip_comments(metadata_size)(input)?;
        let (input, bounding_box) = skip_comments(metadata_bounding_box)(input)?;
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
}

fn metadata_version(input: &str) -> IResult<&str, f32> {
    map_res(statement("STARTFONT", String::parse), |text| text.parse())(input)
}

fn metadata_name(input: &str) -> IResult<&str, String> {
    statement("FONT", String::parse)(input)
}

fn metadata_size(input: &str) -> IResult<&str, (i32, (u32, u32))> {
    statement(
        "SIZE",
        separated_pair(
            i32::parse,
            space1,
            map(Size::parse, |size| (size.width, size.height)),
        ),
    )(input)
}

fn metadata_bounding_box(input: &str) -> IResult<&str, BoundingBox> {
    statement("FONTBOUNDINGBOX", BoundingBox::parse)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::geometry::{Point, Size};

    #[test]
    fn it_parses_the_font_version() {
        assert_eq!(metadata_version("STARTFONT 2.1\n"), Ok(("", 2.1f32)));

        // Some fonts are a bit overzealous with their whitespace
        assert_eq!(metadata_version("STARTFONT   2.1\n"), Ok(("", 2.1f32)));
    }

    #[test]
    fn it_parses_header() {
        let input = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 75
FONTBOUNDINGBOX 16 24 0 0"#;

        assert_eq!(
            Metadata::parse(input),
            Ok((
                "",
                Metadata {
                    version: 2.1,
                    name: String::from("\"test font\""),
                    point_size: 16,
                    resolution: (75, 75),
                    bounding_box: BoundingBox::new(Size::new(16, 24), Point::zero())
                }
            ))
        );
    }
}
