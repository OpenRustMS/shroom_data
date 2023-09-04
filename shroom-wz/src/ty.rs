use std::{
    io::{Read, Seek},
    num::Wrapping,
    ops::Neg,
};

use binrw::{binrw, BinRead, BinWrite, VecArgs};
use image::EncodableLayout;

use crate::{crypto::WzCrypto, util::WzContext};

pub type RefWzCrypto<'a> = (&'a WzCrypto,);

/// Compressed Int
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WzInt(pub i32);

impl BinRead for WzInt {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        Ok(Self(match i8::read_options(reader, endian, args)? {
            -128 => i32::read_options(reader, endian, args)?,
            flag => flag as i32,
        }))
    }
}

impl BinWrite for WzInt {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match i8::try_from(self.0) {
            Ok(v) if v != -128 => v.write_options(writer, endian, args),
            _ => {
                (-128i8).write_options(writer, endian, args)?;
                (self.0).write_options(writer, endian, args)
            }
        }
    }
}

/// Compressed Long
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WzLong(pub i64);

impl BinRead for WzLong {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        Ok(Self(match i8::read_options(reader, endian, args)? {
            -128 => i64::read_options(reader, endian, args)?,
            flag => flag as i64,
        }))
    }
}

impl BinWrite for WzLong {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match i8::try_from(self.0) {
            Ok(v) if v != -128 => v.write_options(writer, endian, args),
            _ => {
                (-128i8).write_options(writer, endian, args)?;
                (self.0).write_options(writer, endian, args)
            }
        }
    }
}

/// Compressed float, converts value to Int value which is compressed
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WzF32(pub f32);

impl BinRead for WzF32 {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        Ok(Self(f32::from_bits(
            WzInt::read_options(reader, endian, args)?.0 as u32,
        )))
    }
}

impl BinWrite for WzF32 {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        WzInt(self.0.to_bits() as i32).write_options(writer, endian, args)
    }
}

// String mask helper

fn xor_mask_ascii(data: &mut [u8]) {
    let mut mask = Wrapping(0xAAu8);
    for b in data.iter_mut() {
        *b ^= mask.0;
        mask += 1;
    }
}

fn xor_mask_unicode(data: &mut [u16]) {
    let mut mask = Wrapping(0xAAAA);
    for b in data.iter_mut() {
        *b ^= mask.0;
        mask += 1;
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct WzOffsetStr {
    pub offset: i32,
}

#[derive(Clone, PartialEq, Eq)]
pub enum WzStr {
    ASCII(Vec<u8>),
    Wide(Vec<u16>),
}

impl std::fmt::Debug for WzStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "x/{:?}", self.as_bytes()),
        }
    }
}

impl WzStr {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::ASCII(s) => s.as_slice(),
            Self::Wide(s) => bytemuck::cast_slice(s.as_slice()),
        }
    }

    pub fn from_ascii(s: &str) -> Self {
        //TODO check ascii
        Self::ASCII(s.as_bytes().to_vec())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::ASCII(s) => std::str::from_utf8(s.as_bytes()).ok(),
            _ => None,
        }
    }

    pub fn as_ascii_str(&self) -> Option<&[u8]> {
        match self {
            Self::ASCII(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    pub fn as_wstr(&self) -> Option<&[u16]> {
        match self {
            Self::Wide(s) => Some(s.as_slice()),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Self::ASCII(s) => std::str::from_utf8(s.as_slice())
                .ok()
                .map(|s| s.to_string()),
            Self::Wide(s) => String::from_utf16(s.as_slice()).ok(),
        }
    }
}

impl BinRead for WzStr {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (is_ascii, len) = {
            let flag = i8::read_options(reader, endian, ())?;
            match flag {
                // Negative is ASCII
                -128 => (true, i32::read_options(reader, endian, ())? as usize),
                ln if ln <= 0 => (true, -ln as usize),
                //Positive is unicode
                127 => (false, i32::read_options(reader, endian, ())? as usize),
                ln => (false, ln as usize),
            }
        };

        Ok(if is_ascii {
            let mut data = vec![0; len];
            reader.read_exact(&mut data)?;
            xor_mask_ascii(&mut data);
            args.crypto.transform(data.as_mut_slice().into());
            WzStr::ASCII(data)
        } else {
            let mut data = vec![0u16; len];
            reader.read_exact(bytemuck::cast_slice_mut(data.as_mut_slice()))?;
            xor_mask_unicode(&mut data);
            args.crypto
                .transform(bytemuck::cast_slice_mut(data.as_mut_slice()).into());

            WzStr::Wide(data)
        })
    }
}

impl BinWrite for WzStr {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            Self::ASCII(ref data) => {
                let n = data.len();
                if n >= 128 {
                    i8::MIN.write_options(writer, endian, ())?;
                    (n as i32).neg().write_options(writer, endian, ())?;
                } else {
                    (n as i8).neg().write_options(writer, endian, ())?;
                }

                let mut data = data.clone();
                args.crypto.transform(data.as_mut_slice().into());
                xor_mask_ascii(&mut data);

                data.write_options(writer, endian, ())?;
            }
            Self::Wide(ref data) => {
                let n = data.len();
                if n >= 127 {
                    i8::MAX.write_options(writer, endian, ())?;
                    (n as i32).write_options(writer, endian, ())?;
                } else {
                    (n as i8).write_options(writer, endian, ())?;
                }

                let mut data = data.clone();
                args.crypto
                    .transform(bytemuck::cast_slice_mut(data.as_mut_slice()).into());
                xor_mask_unicode(&mut data);
                data.as_bytes().write_options(writer, endian, ())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WzVec<B>(pub Vec<B>);

impl<B> BinRead for WzVec<B>
where
    B: BinRead + 'static,
    for<'a> B::Args<'a>: Clone,
{
    type Args<'a> = B::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let n = WzInt::read_options(reader, endian, ())?;
        Ok(Self(Vec::read_options(
            reader,
            endian,
            VecArgs {
                count: n.0 as usize,
                inner: args,
            },
        )?))
    }
}

impl<B> BinWrite for WzVec<B>
where
    B: BinWrite + 'static,
    for<'a> B::Args<'a>: Clone,
{
    type Args<'a> = B::Args<'a>;

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        WzInt(self.0.len() as i32).write_options(writer, endian, ())?;
        self.0.write_options(writer, endian, args)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WzOffset(pub u32);

impl From<WzOffset> for u32 {
    fn from(value: WzOffset) -> Self {
        value.0
    }
}

impl From<WzOffset> for u64 {
    fn from(value: WzOffset) -> u64 {
        value.0 as u64
    }
}

impl BinRead for WzOffset {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()? as u32;
        let v = u32::read_options(reader, endian, ())?;
        let offset = args.crypto.decrypt_offset(v, pos);
        Ok(Self(offset))
    }
}

impl BinWrite for WzOffset {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        let pos = writer.stream_position()? as u32;
        let enc_off = args.crypto.encrypt_offset(self.0, pos);
        enc_off.write_options(writer, endian, ())
    }
}
