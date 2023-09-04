pub mod tree;
use std::io;

use crate::util::WzContext;
use binrw::{binrw, BinRead, BinWrite, NullString};

use crate::ty::{WzInt, WzOffset, WzStr, WzVec};

#[binrw]
#[brw(little)]
#[br(magic = b"PKG1")]
#[derive(Debug)]
pub struct WzHeader {
    pub file_size: u64,
    pub data_offset: u32,
    pub desc: NullString,
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug, Clone)]
pub struct WzDir {
    #[brw(args_raw(ctx))]
    pub entries: WzVec<WzDirNode>,
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WzImgHeader {
    #[brw(args_raw(ctx))]
    pub name: WzStr,
    pub blob_size: WzInt,
    pub checksum: WzInt,
    #[brw(args_raw(ctx))]
    pub offset: WzOffset,
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WzDirHeader {
    #[brw(args_raw(ctx))]
    pub name: WzStr,
    pub blob_size: WzInt,
    pub checksum: WzInt,
    #[brw(args_raw(ctx))]
    pub offset: WzOffset,
}

impl WzDirHeader {
    pub fn root(root_size: usize, offset: WzOffset) -> Self {
        Self {
            name: WzStr::from_ascii("Root"),
            blob_size: WzInt(root_size as i32),
            checksum: WzInt(1),
            offset: offset,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WzLinkData {
    pub offset: u32,
    pub ty: u8,
    pub name: WzStr,
}

impl BinRead for WzLinkData {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let offset = u32::read_options(reader, endian, ())?;
        let old_pos = reader.stream_position()?;
        reader.seek(io::SeekFrom::Start(args.crypto.offset_link(offset)))?;

        dbg!(args.crypto.offset_link(offset));
        dbg!(offset);

        let ty = u8::read_options(reader, endian, ())?;
        let name = WzStr::read_options(reader, endian, args)?;
        dbg!(&name);

        // Seek back
        reader.seek(io::SeekFrom::Start(old_pos))?;

        Ok(Self { offset, ty, name })
    }
}

impl BinWrite for WzLinkData {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        todo!()
    }
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug, Clone, PartialEq)]
pub struct WzLinkHeader {
    #[brw(args_raw(ctx))]
    pub link: WzLinkData,
    pub blob_size: WzInt,
    pub checksum: WzInt,
    #[brw(args_raw(ctx))]
    pub offset: WzOffset,
}

#[derive(BinRead, BinWrite, Debug, Clone, PartialEq)]
#[brw(little, import_raw(ctx: WzContext<'_>))]
pub enum WzDirNode {
    //01 XX 00 00 00 00 00 OFFSET (4 bytes)
    #[br(magic(1u8))]
    Nil([u8; 10]),
    #[br(magic(2u8))]
    Link(#[brw(args_raw(ctx))] WzLinkHeader),
    #[br(magic(3u8))]
    Dir(#[brw(args_raw(ctx))] WzDirHeader),
    #[brw(magic(4u8))]
    Img(#[brw(args_raw(ctx))] WzImgHeader),
}