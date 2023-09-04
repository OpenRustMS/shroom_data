pub mod obj;
pub mod tree;
use binrw::{binread, binrw, BinWrite, FilePtr};

use crate::ty::WzStr;
use crate::util::WzContext;

pub mod canvas;
pub mod prop;
pub mod sound;

#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct WzOffsetStr {
    pub offset: i32,
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
    OffsetTypeName(#[brw(args { inner: ctx })] FilePtr<u32, WzStr>),
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
            Self::OffsetTypeName(arg0) => Self::OffsetTypeName(FilePtr {
                ptr: arg0.ptr,
                value: arg0.value.clone(),
            }),
        }
    }
}

impl WzUOLStr {
    pub fn as_ref(&self) -> &WzStr {
        match self {
            Self::Str(s) | Self::StrTypeName(s) => s,
            Self::Offset(s) | Self::OffsetTypeName(s) => s.value.as_ref().unwrap(),
        }
    }

    pub fn to_string(&self) -> Option<String> {
        self.as_ref().to_string()
    }
    pub fn as_str(&self) -> Option<&str> {
        self.as_ref().as_str()
    }

    pub fn as_ascii_str(&self) -> Option<&[u8]> {
        self.as_ref().as_ascii_str()
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
