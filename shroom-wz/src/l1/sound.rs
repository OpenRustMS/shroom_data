use std::io::Cursor;

use binrw::{binrw, BinRead, BinReaderExt, BinWrite, PosValue};

use crate::{ty::WzInt, util::WzContext};

// TODO verify paddings

const WAVE_HEADER_SIZE: usize = 18;
const PCM_HEADER_SIZE: usize = 44;

const WAVE_FORMAT_PCM: u16 = 0x0001;
const WAVE_FORMAT_MP3: u16 = 0x0055;

#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct GUID(pub [u8; 16]);

// See WAVEFORMATEX
// https://learn.microsoft.com/en-us/windows/win32/api/mmeapi/ns-mmeapi-waveformatex
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct WaveHeader {
    pub format: u16,
    pub channels: u16,
    pub samples_per_sec: u32,
    pub avg_bytes_per_sec: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    // Align tail, struct is not packed
    #[bw(pad_size_to = 4)]
    pub extra_size: u16,
}

impl WaveHeader {
    pub fn is_valid_header_size(&self, header_size: usize) -> bool {
        WAVE_HEADER_SIZE + (self.extra_size as usize) == header_size
    }
}

// see MPEGLAYER3WAVEFORMAT
// https://learn.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-mpeglayer3waveformat
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct Mpeg3WaveHeader {
    pub wav: WaveHeader,
    #[bw(pad_size_to = 4)]
    pub id: u16,
    pub flags: u32,
    pub block_size: u16,
    pub frames_per_block: u16,
    #[bw(pad_size_to = 4)]
    pub codec_delay: u16,
}

//PCMWAVEFORMAT
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct PcmWaveFormat {
    pub wav: WaveHeader,
    #[bw(pad_size_to = 4)]
    pub bit_per_sample: u16,
}

#[derive(Debug)]
pub enum SoundFormat {
    Mpeg3(Mpeg3WaveHeader),
    Pcm(WaveHeader),
}

impl BinRead for SoundFormat {
    type Args<'a> = WzContext<'a>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let hdr_len = u8::read_le(reader)? as usize;

        // Header is atmost u8::MAX, read the header
        let mut buf = [0; u8::MAX as usize];
        let hdr_buf = &mut buf[..hdr_len];
        reader.read_exact(hdr_buf)?;

        // Build reader
        let mut wave: WaveHeader = Cursor::new(&hdr_buf).read_le()?;

        // Check if wave header looks valid, else wise try to decode
        if !wave.is_valid_header_size(hdr_len) {
            args.crypto.transform(hdr_buf.into());
            wave = Cursor::new(&hdr_buf).read_le()?;

            if !wave.is_valid_header_size(hdr_len) {
                todo!("Invalid header size")
            }
        }

        // We got our wave header now check the extra data
        Ok(match wave.format {
            WAVE_FORMAT_PCM => SoundFormat::Pcm(wave),
            WAVE_FORMAT_MP3 => Self::Mpeg3(Cursor::new(&hdr_buf).read_le()?),
            n => todo!("Invalid wave format: {n}"),
        })
    }
}

impl BinWrite for SoundFormat {
    type Args<'a> = WzContext<'a>;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        // Encode it
        match self {
            Self::Mpeg3(mp3) => {
                (WAVE_HEADER_SIZE as u8 + mp3.wav.extra_size as u8).write_le(writer)?;
                mp3.write_args(writer, ())
            }
            Self::Pcm(wave) => {
                (WAVE_HEADER_SIZE as u8 + wave.extra_size as u8).write_le(writer)?;
                wave.write_args(writer, ())
            }
        }
    }
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug)]
pub struct SoundHeader {
    pub unknown1: u8, // 1
    pub major_id: GUID,
    pub sub_type: GUID,
    pub unknown2: u16, //32 +2 +1
    pub id: GUID,      // 35 + 16
    #[brw(args_raw = ctx)]
    pub fmt: SoundFormat,
}

#[binrw]
#[brw(little, import_raw(ctx: WzContext<'_>))]
#[derive(Debug)]
pub struct WzSound {
    pub unknown: u8,
    pub size: WzInt,
    pub len_ms: WzInt,
    #[brw(args_raw = ctx)]
    pub header: SoundHeader,
    #[bw(ignore)]
    pub offset: PosValue<()>,
}

impl WzSound {
    pub fn data_size(&self) -> usize {
        let extra = match self.header.fmt {
            SoundFormat::Mpeg3(_) => 0,
            SoundFormat::Pcm(_) => PCM_HEADER_SIZE,
        };
        (self.size.0 as usize) + extra
    }
}
