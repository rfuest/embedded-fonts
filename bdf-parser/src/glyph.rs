use embedded_graphics::prelude::*;
use nom::{
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::multispace0,
    combinator::{map, map_parser, map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use std::convert::TryFrom;

use super::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub name: String,
    pub encoding: Option<char>,
    pub bounding_box: BoundingBox,
    pub bitmap: Vec<u8>,
    pub scalable_width: Option<Size>,
    pub device_width: Option<Size>,
}

impl Parse for Glyph {
    fn parse(input: &str) -> IResult<&str, Glyph> {
        let (input, name) = statement("STARTCHAR", String::parse)(input)?;
        let (input, encoding) = statement("ENCODING", parse_encoding)(input)?;
        let (input, scalable_width) = opt(statement("SWIDTH", Size::parse))(input)?;
        let (input, device_width) = opt(statement("DWIDTH", Size::parse))(input)?;
        let (input, bounding_box) = statement("BBX", BoundingBox::parse)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, bitmap) = parse_bitmap(input)?;

        Ok((
            input,
            Glyph {
                bitmap,
                bounding_box,
                encoding,
                name,
                scalable_width,
                device_width,
            },
        ))
    }
}

fn hex_byte(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit()), |v| {
        u8::from_str_radix(v, 16)
    })(input)
}

fn parse_bitmap(input: &str) -> IResult<&str, Vec<u8>> {
    map_parser(
        delimited(tag("BITMAP"), take_until("ENDCHAR"), tag("ENDCHAR")),
        preceded(multispace0, many0(terminated(hex_byte, multispace0))),
    )(input)
}

fn parse_encoding(input: &str) -> IResult<&str, Option<char>> {
    map(i32::parse, |code| {
        u32::try_from(code).ok().and_then(std::char::from_u32)
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::prelude::{Point, Size};

    #[test]
    fn it_parses_bitmap_data() {
        assert_eq!(parse_bitmap("BITMAP\n7e\nENDCHAR"), Ok(("", vec![0x7e])));
        assert_eq!(parse_bitmap("BITMAP\nff\nENDCHAR"), Ok(("", vec![0xff])));
        assert_eq!(
            parse_bitmap("BITMAP\nCCCC\nENDCHAR"),
            Ok(("", vec![0xcc, 0xcc]))
        );
        assert_eq!(
            parse_bitmap("BITMAP\nffffffff\nENDCHAR"),
            Ok(("", vec![0xff, 0xff, 0xff, 0xff]))
        );
        assert_eq!(
            parse_bitmap("BITMAP\nffffffff\naaaaaaaa\nENDCHAR"),
            Ok(("", vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa]))
        );
        assert_eq!(
            parse_bitmap("BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR"),
            Ok(("", vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa]))
        );
        assert_eq!(
            parse_bitmap(
                "BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
                    .as_ref()
            ),
            Ok((
                "",
                vec![
                    0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42,
                    0x42, 0x00, 0x00
                ]
            ))
        );
    }

    #[test]
    fn it_parses_a_single_char() {
        let chardata = r#"STARTCHAR ZZZZ
ENCODING 65
SWIDTH 500 0
DWIDTH 8 0
BBX 8 16 0 -2
BITMAP
00
00
00
00
18
24
24
42
42
7E
42
42
42
42
00
00
ENDCHAR"#;

        assert_eq!(
            Glyph::parse(chardata),
            Ok((
                "",
                Glyph {
                    name: "ZZZZ".to_string(),
                    encoding: Some('A'), //65
                    bitmap: vec![
                        0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42,
                        0x42, 0x42, 0x00, 0x00
                    ],
                    bounding_box: BoundingBox::new(Size::new(8, 16), Point::new(0, -2)),
                    scalable_width: Some(Size::new(500, 0)),
                    device_width: Some(Size::new(8, 0)),
                }
            ))
        );
    }

    #[test]
    fn it_parses_negative_encodings() {
        let chardata = r#"STARTCHAR 000
ENCODING -1
SWIDTH 432 0
DWIDTH 6 0
BBX 0 0 0 0
BITMAP
ENDCHAR"#;

        assert_eq!(
            Glyph::parse(chardata),
            Ok((
                "",
                Glyph {
                    bitmap: vec![],
                    bounding_box: BoundingBox::new(Size::zero(), Point::zero()),
                    encoding: None,
                    name: "000".to_string(),
                    scalable_width: Some(Size::new(432, 0)),
                    device_width: Some(Size::new(6, 0)),
                }
            ))
        );
    }

    #[test]
    fn it_parses_chars_with_no_bitmap() {
        let chardata = r#"STARTCHAR 000
ENCODING 0
SWIDTH 432 0
DWIDTH 6 0
BBX 0 0 0 0
BITMAP
ENDCHAR"#;

        assert_eq!(
            Glyph::parse(chardata),
            Ok((
                "",
                Glyph {
                    bitmap: vec![],
                    bounding_box: BoundingBox::new(Size::zero(), Point::zero()),
                    encoding: Some('\x00'),
                    name: "000".to_string(),
                    scalable_width: Some(Size::new(432, 0)),
                    device_width: Some(Size::new(6, 0)),
                }
            ))
        );
    }
}
