use embedded_graphics::{prelude::*, primitives::Rectangle};
use nom::{
    bytes::complete::{tag, take_until},
    character::{complete::multispace0, is_hex_digit},
    combinator::opt,
    sequence::delimited,
    IResult,
};

use super::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub name: String,
    pub charcode: i32,
    pub bounding_box: Rectangle,
    pub bitmap: Vec<u32>,
    pub scalable_width: Option<Size>,
    pub device_width: Option<Size>,
}

fn glyph_bitmap(input: &[u8]) -> IResult<&[u8], Vec<u32>> {
    let (input, _) = multispace0(input)?;
    let (input, glyph_data) =
        delimited(tag("BITMAP"), take_until("ENDCHAR"), tag("ENDCHAR"))(input)?;

    Ok((
        input,
        glyph_data
            .to_vec()
            .iter()
            .filter(|d| is_hex_digit(**d))
            .collect::<Vec<&u8>>()
            .chunks(8)
            .map(|c| {
                c.iter()
                    .rev()
                    .enumerate()
                    .map(|(k, &&v)| {
                        let digit = v as char;
                        digit.to_digit(16).unwrap_or(0) << (k * 4)
                    })
                    .sum()
            })
            .collect(),
    ))
}

pub fn glyph(input: &[u8]) -> IResult<&[u8], Glyph> {
    let (input, name) = statement("STARTCHAR", String::parse)(input)?;
    let (input, charcode) = statement("ENCODING", i32::parse)(input)?;
    let (input, scalable_width) = opt(statement("SWIDTH", Size::parse))(input)?;
    let (input, device_width) = opt(statement("DWIDTH", Size::parse))(input)?;
    let (input, bounding_box) = statement("BBX", Rectangle::parse)(input)?;
    let (input, bitmap) = glyph_bitmap(input)?;

    Ok((
        input,
        Glyph {
            bitmap,
            bounding_box,
            charcode,
            name,
            scalable_width,
            device_width,
        },
    ))
}

#[cfg(test)]
mod tests {
    use embedded_graphics::prelude::{Point, Size};

    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_bitmap_data() {
        assert_eq!(
            glyph_bitmap(b"BITMAP\n7e\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0x7e]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nff\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![255]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nCCCC\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xcccc]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nffffffff\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(
                b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
                    .as_ref()
            ),
            Ok((EMPTY, vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000]))
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

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    name: "ZZZZ".to_string(),
                    charcode: 65,
                    bitmap: vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000],
                    bounding_box: Rectangle::new(Point::new(0, -2), Size::new(8, 16)),
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

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: Rectangle::new(Point::zero(), Size::zero()),
                    charcode: -1i32,
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

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: Rectangle::new(Point::zero(), Size::zero()),
                    charcode: 0,
                    name: "000".to_string(),
                    scalable_width: Some(Size::new(432, 0)),
                    device_width: Some(Size::new(6, 0)),
                }
            ))
        );
    }
}
