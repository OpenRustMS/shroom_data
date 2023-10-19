pub mod obj;
pub mod tree;
use std::rc::Rc;

use binrw::{binread, BinRead, BinWrite, FilePtr};

use crate::ty::WzStr;
use crate::util::WzContext;

pub mod canvas;
pub mod prop;
pub mod ser;
pub mod sound;

#[derive(Debug, Clone)]
pub struct WzOffsetStr(Rc<WzStr>);

impl BinRead for WzOffsetStr {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        // Read the offset
        let off = u32::read_options(reader, endian, ())?;

        let str = args.read_offset_str(reader, off).unwrap(); // TODO err
        Ok(Self(str))
    }
}

impl BinWrite for WzOffsetStr {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        _writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        todo!()
    }
}

#[binread]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug)]
pub enum WzUOLStr {
    #[br(magic(0u8))]
    Str(#[brw(args_raw(ctx))] WzStr),
    #[br(magic(0x73u8))]
    StrTypeName(#[brw(args_raw(ctx))] WzStr),
    #[br(magic(1u8))]
    Offset(#[brw(args { inner: ctx })] FilePtr<u32, WzStr>),
    #[br(magic(0x1bu8))]
    OffsetTypeName(#[brw(args_raw(ctx))] WzOffsetStr),
}

impl Clone for WzUOLStr {
    fn clone(&self) -> Self {
        match self {
            Self::Str(arg0) => Self::Str(arg0.clone()),
            Self::StrTypeName(arg0) => Self::StrTypeName(arg0.clone()),
            Self::Offset(arg0) => Self::Offset(FilePtr {
                ptr: arg0.ptr,
                value: arg0.value.clone(),
            }),
            Self::OffsetTypeName(v) => Self::OffsetTypeName(v.clone()),
        }
    }
}

impl AsRef<WzStr> for WzUOLStr {
    fn as_ref(&self) -> &WzStr {
        match self {
            Self::Str(s) | Self::StrTypeName(s) => s,
            Self::OffsetTypeName(s) => s.0.as_ref(),
            Self::Offset(s) => &s.value,
        }
    }
}

impl std::fmt::Display for WzUOLStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl BinWrite for WzUOLStr {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        _writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        todo!()
    }
}
