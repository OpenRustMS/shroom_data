use binrw::{BinRead, BinWrite};

use crate::util::WzContext;

use super::{
    canvas::WzCanvas,
    prop::{WzConvex2D, WzProperty, WzUOL, WzVector2D},
    sound::WzSound,
    WzUOLStr,
};

#[derive(Debug)]
pub enum WzObject {
    Property(WzProperty),
    Canvas(WzCanvas),
    UOL(WzUOL),
    Vec2(WzVector2D),
    Convex2D(WzConvex2D),
    SoundDX8(WzSound),
}

impl BinRead for WzObject {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let ty_name = WzUOLStr::read_options(reader, endian, args)?;

        let ty_name = ty_name.as_ascii_str().ok_or_else(|| binrw::Error::Custom {
            pos: 0,
            err: Box::new(anyhow::format_err!("Invalid type name")),
        })?;

        Ok(match ty_name {
            b"Property" => Self::Property(WzProperty::read_options(reader, endian, args)?),
            b"Canvas" => Self::Canvas(WzCanvas::read_options(reader, endian, args)?),
            b"UOL" => Self::UOL(WzUOL::read_options(reader, endian, args)?),
            b"Shape2D#Vector2D" => Self::Vec2(WzVector2D::read_options(reader, endian, ())?),
            b"Shape2D#Convex2D" => Self::Convex2D(WzConvex2D::read_options(reader, endian, args)?),
            b"Sound_DX8" => Self::SoundDX8(WzSound::read_options(reader, endian, args)?),
            _ => {
                return Err(binrw::Error::Custom {
                    pos: reader.stream_position().unwrap_or(0),
                    err: Box::new(anyhow::format_err!("Invalid obj: {ty_name:?}")),
                })
            }
        })
    }
}

impl BinWrite for WzObject {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            WzObject::Property(v) => v.write_options(writer, endian, args),
            WzObject::UOL(v) => v.write_options(writer, endian, args),
            WzObject::Vec2(v) => v.write_options(writer, endian, ()),
            WzObject::Convex2D(v) => v.write_options(writer, endian, args),
            _ => todo!(), /*
                          WzObject::Canvas(v) => v.write_options(writer, endian, (args,)),
                          WzObject::SoundDX8(v) => v.write_options(writer, endian, (args,)),
                          */
        }
    }
}
