use binrw::{binread, PosValue};

use crate::ty::WzInt;
use crate::util::WzContext;

use super::prop::WzProperty;

#[derive(Debug)]
pub struct WzCanvasScaling(pub u8);

impl WzCanvasScaling {
    pub fn get_factor(&self) -> u32 {
        // 2_pow(scale)
        2 >> self.0
    }
}

impl TryFrom<u8> for WzCanvasScaling {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let n = value;
        Ok(Self(match n {
            0 | 4 => n,
            _ => anyhow::bail!("Invalid scaling: {n}"),
        }))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WzCanvasDepth {
    BGRA4444,
    BGRA8888,
    BGR565,
    DXT3,
    DXT5,
}

impl WzCanvasDepth {
    pub fn depth_size(&self) -> u32 {
        match self {
            WzCanvasDepth::BGRA4444 => 2,
            WzCanvasDepth::BGRA8888 => 4,
            WzCanvasDepth::BGR565 => 2,
            WzCanvasDepth::DXT3 => 1,
            WzCanvasDepth::DXT5 => 1,
        }
    }
}

impl TryFrom<WzInt> for WzCanvasDepth {
    type Error = anyhow::Error;

    fn try_from(value: WzInt) -> Result<Self, Self::Error> {
        Ok(match value.0 as u16 {
            1 => Self::BGRA4444,
            2 => Self::BGRA8888,
            513 => Self::BGR565,
            1026 => Self::DXT3,
            2050 => Self::DXT5,
            depth => anyhow::bail!("Invalid canvas depth: {depth}"),
        })
    }
}

#[binread]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug)]
pub struct WzCanvas {
    pub unknown: u8,
    pub has_property: u8,
    #[br(if(has_property.eq(&1)), args_raw(ctx))]
    pub property: Option<WzProperty>,
    pub width: WzInt,
    pub height: WzInt,
    #[br(try_map = |x: WzInt| x.try_into())]
    pub depth: WzCanvasDepth,
    #[br(try_map = |x: u8| x.try_into())]
    pub scale: WzCanvasScaling,
    pub unknown1: u32,
    pub len: PosValue<u32>,
}

impl WzCanvas {
    pub fn pixels(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn height(&self) -> u32 {
        self.height.0 as u32
    }

    pub fn width(&self) -> u32 {
        self.width.0 as u32
    }

    pub fn bitmap_size(&self) -> u32 {
        self.pixels() * self.depth.depth_size()
    }

    pub fn scaled_pixels(&self) -> u32 {
        self.scaled_height() * self.scaled_width()
    }

    pub fn scaled_height(&self) -> u32 {
        self.height() / self.scale.get_factor()
    }

    pub fn scaled_width(&self) -> u32 {
        self.width() / self.scale.get_factor()
    }

    pub fn scaled_bitmap_size(&self) -> u32 {
        self.scaled_pixels() * self.depth.depth_size()
    }

    pub fn data_len(&self) -> usize {
        self.len.val as usize - 1
    }

    pub fn data_offset(&self) -> u64 {
        self.len.pos + 4 + 1
    }
}
