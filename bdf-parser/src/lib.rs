extern crate nom;

use nom::{
    bytes::complete::tag, character::complete::multispace0, combinator::opt, multi::many0, IResult,
};

mod glyph;
mod helpers;
mod metadata;
mod properties;

use glyph::*;
use helpers::*;
use metadata::*;
use properties::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BDFFont {
    metadata: Option<Metadata>,
    glyphs: Vec<Glyph>,
    properties: Option<Properties>,
}

pub struct BDFParser<'a> {
    source: &'a str,
}

impl<'a> BDFParser<'a> {
    pub fn from_str(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(&self) -> IResult<&[u8], BDFFont> {
        bdf(&self.source.as_bytes())
    }
}

fn bdf(input: &[u8]) -> IResult<&[u8], BDFFont> {
    let (input, metadata) = opt(header)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, properties) = opt(properties)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(statement("CHARS", u32::parse))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, glyphs) = many0(glyph)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(tag("ENDFONT"))(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        BDFFont {
            properties,
            metadata,
            glyphs,
        },
    ))
}

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
mod tests {
    use embedded_graphics::{prelude::Point, prelude::Size, primitives::Rectangle};

    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_a_font_file() {
        let chardata = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 75
FONTBOUNDINGBOX 16 24 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
ENDFONT
"#;

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                BDFFont {
                    metadata: Some(Metadata {
                        version: 2.1,
                        name: String::from("\"test font\""),
                        point_size: 16,
                        resolution: (75, 75),
                        bounding_box: Rectangle::new(Point::zero(), Size::new(16, 24)),
                    }),
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: Rectangle::new(Point::zero(), Size::new(8, 8)),
                            charcode: 64,
                            name: "000".to_string(),
                            scalable_width: None,
                            device_width: Some(Size::new(8, 0)),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: Rectangle::new(Point::zero(), Size::new(8, 8)),
                            charcode: 64,
                            name: "000".to_string(),
                            scalable_width: None,
                            device_width: Some(Size::new(8, 0)),
                        },
                    ],
                    properties: Some(hashmap! {
                        "COPYRIGHT".into() => PropertyValue::Text("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".into()),
                        "FONT_ASCENT".into() => PropertyValue::Int(0),
                        "FONT_DESCENT".into() => PropertyValue::Int(0),
                    })
                }
            ))
        );
    }

    #[test]
    fn it_parses_optional_endfont_tag() {
        let chardata = r#"STARTFONT 2.1
FONT "open_iconic_all_1x"
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
"#;

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                BDFFont {
                    metadata: Some(Metadata {
                        version: 2.1,
                        name: String::from("\"open_iconic_all_1x\""),
                        point_size: 16,
                        resolution: (75, 75),
                        bounding_box: Rectangle::new(Point::zero(), Size::new(16, 16)),
                    }),
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: Rectangle::new(Point::zero(), Size::new(8, 8)),
                            charcode: 64,
                            name: "000".to_string(),
                            scalable_width: None,
                            device_width: Some(Size::new(8, 0)),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: Rectangle::new(Point::zero(), Size::new(8, 8)),
                            charcode: 64,
                            name: "000".to_string(),
                            scalable_width: None,
                            device_width: Some(Size::new(8, 0)),
                        },
                    ],
                    properties: Some(hashmap! {
                        "COPYRIGHT".into() => PropertyValue::Text("https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE".into()),
                        "FONT_ASCENT".into() => PropertyValue::Int(0),
                        "FONT_DESCENT".into() => PropertyValue::Int(0),
                    })
                }
            ))
        );
    }

    #[test]
    fn it_handles_windows_line_endings() {
        let windows_line_endings = "STARTFONT 2.1\r\nFONT \"windows_test\"\r\nSIZE 10 96 96\r\nFONTBOUNDINGBOX 8 16 0 -4\r\nCHARS 256\r\nSTARTCHAR 0\r\nENCODING 0\r\nSWIDTH 600 0\r\nDWIDTH 8 0\r\nBBX 8 16 0 -4\r\nBITMAP\r\nD5\r\nENDCHAR\r\nENDFONT\r\n";
        let out = bdf(&windows_line_endings.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                BDFFont {
                    metadata: Some(Metadata {
                        version: 2.1,
                        name: String::from("\"windows_test\""),
                        point_size: 10,
                        resolution: (96, 96),
                        bounding_box: Rectangle::new(Point::new(0, -4), Size::new(8, 16)),
                    }),
                    glyphs: vec![Glyph {
                        bitmap: vec![0xd5],
                        bounding_box: Rectangle::new(Point::new(0, -4), Size::new(8, 16)),
                        charcode: 0,
                        name: "0".to_string(),
                        scalable_width: Some(Size::new(600, 0)),
                        device_width: Some(Size::new(8, 0)),
                    },],
                    properties: None
                }
            ))
        );
    }
}
