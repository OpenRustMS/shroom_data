use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use binrw::BinRead;

use crate::{
    canvas::Canvas,
    crypto::WzCrypto,
    l0::{WzDir, WzDirHeader, WzDirNode, WzHeader, WzImgHeader},
    l1::{
        canvas::WzCanvas,
        obj::WzObject,
        prop::{WzObj, WzValue},
    },
    ty::WzOffset,
    util::{BufReadExt, SubReader},
    version::{WzRegion, WzVersion},
};
pub trait WzIO: BufRead + Seek {}
impl<T> WzIO for T where T: BufRead + Seek {}

pub struct WzImgReader<R> {
    r: R,
    crypto: Rc<WzCrypto>,
}

impl<R> WzImgReader<R>
where
    R: WzIO,
{
    pub fn read_path(&mut self, root: &WzObject, path: &str) -> anyhow::Result<WzObject> {
        let mut cur = root.clone();
        let mut obj_storage = None;

        for part in path.split('/') {
            let WzObject::Property(ref prop) = cur else {
                anyhow::bail!("Invalid prop: {cur:?}");
            };

            let next = prop
                .entries
                .0
                .iter()
                .find(|x| x.name.as_str() == Some(part))
                .ok_or_else(|| anyhow::format_err!("Invalid {path}"))?;

            let obj = match &next.val {
                WzValue::Obj(ref obj) => obj,
                _ => anyhow::bail!("Invalid obj: {cur:?}"),
            };
            obj_storage = Some(self.read_obj(&obj)?);
            cur = obj_storage.as_ref().unwrap();
        }

        obj_storage.ok_or_else(|| anyhow::format_err!("Invalid {path}"))
    }

    /// Read the root object for that image
    pub fn read_root_obj(&mut self) -> anyhow::Result<WzObject> {
        self.r.rewind()?;
        Ok(WzObject::read_le_args(
            &mut self.r,
            self.crypto.as_ref().into(),
        )?)
    }

    /// Read an object with the given object header
    pub fn read_obj(&mut self, obj: &WzObj) -> anyhow::Result<WzObject> {
        // Skip first index
        self.r.seek(SeekFrom::Start(obj.len.pos + 4))?;
        Ok(WzObject::read_le_args(
            &mut self.r,
            self.crypto.as_ref().into(),
        )?)
    }

    fn read_canvas_from<T: BufRead>(mut r: T, canvas: &WzCanvas) -> anyhow::Result<Canvas> {
        let mut img_buf = Vec::with_capacity(canvas.bitmap_size() as usize);
        r.decompress_flate(&mut img_buf)?;
        Ok(Canvas::from_data(img_buf, canvas))
    }

    pub fn read_canvas(&mut self, canvas: &WzCanvas) -> anyhow::Result<Canvas> {
        let len = canvas.data_len();
        let off = canvas.data_offset();
        self.r.seek(SeekFrom::Start(off))?;

        let hdr = self.r.peek_u16()?;
        // Match header
        match hdr {
            0x9C78 | 0xDA78 | 0x0178 | 0x5E78 => {
                let mut sub = (&mut self.r).take(len as u64);
                Self::read_canvas_from(&mut sub, canvas)
                // TODO maybe advance r here not sure
            }
            _ => {
                let buf = self.r.read_chunked_data(&self.crypto, len)?;
                Self::read_canvas_from(Cursor::new(buf), canvas)
            }
        }
    }
}

#[derive(Debug)]
pub struct WzReader<R> {
    inner: R,
    crypto: Rc<WzCrypto>,
    data_offset: u64,
}

pub type SubWzReader<'a, R> = WzReader<SubReader<'a, R>>;
pub type WzReaderFile = WzReader<BufReader<File>>;

impl WzReaderFile {
    pub fn open_file(
        path: impl AsRef<Path>,
        region: WzRegion,
        version: WzVersion,
    ) -> anyhow::Result<Self> {
        Self::open(BufReader::new(File::open(path)?), region, version)
    }
}

impl<R> WzReader<R>
where
    R: WzIO,
{
    pub fn open(mut rdr: R, region: WzRegion, ver: WzVersion) -> anyhow::Result<Self> {
        let hdr = WzHeader::read_le(&mut rdr)?;
        rdr.seek(SeekFrom::Start(hdr.data_offset as u64))?;

        let encrypted_version = u16::read_le(&mut rdr)?;
        if ver.encrypted_version() != encrypted_version {
            anyhow::bail!("Wrong version: {}, expected: {ver:?}", encrypted_version);
        }

        Ok(Self {
            inner: rdr,
            crypto: WzCrypto::from_region(region, ver, hdr.data_offset).into(),
            data_offset: hdr.data_offset as u64,
        })
    }

    fn sub_reader(&mut self, offset: u64, size: u64) -> SubReader<'_, R> {
        SubReader::new(&mut self.inner, offset, size)
    }

    pub fn read_root_dir(&mut self) -> anyhow::Result<WzDir> {
        // Skip encrypted version at the start
        self.read_dir(self.data_offset + 2)
    }

    pub fn read_dir_node(&mut self, hdr: &WzDirHeader) -> anyhow::Result<WzDir> {
        self.read_dir(hdr.offset.0 as u64)
    }

    fn read_dir(&mut self, offset: u64) -> anyhow::Result<WzDir> {
        self.set_pos(offset)?;
        Ok(WzDir::read_le_args(
            &mut self.inner,
            (self.crypto.as_ref()).into(),
        )?)
    }

    pub fn img_reader(&mut self, hdr: &WzImgHeader) -> io::Result<WzImgReader<SubReader<'_, R>>> {
        let off = hdr.offset.into();
        self.set_pos(off)?;
        let crypto = self.crypto.clone();

        Ok(WzImgReader {
            r: self.sub_reader(off, hdr.blob_size.0 as u64),
            crypto,
        })
    }

    pub fn img_iter(&mut self) -> WzImgIter<'_, R> {
        let mut q = VecDeque::new();
        q.push_back((
            Arc::new("".to_string()),
            WzDirNode::Dir(WzDirHeader::root(1, self.root_offset())),
        ));
        WzImgIter { r: self, q }
    }

    pub fn root_offset(&self) -> WzOffset {
        WzOffset(self.data_offset as u32 + 2)
    }

    fn set_pos(&mut self, p: u64) -> io::Result<()> {
        self.inner.seek(SeekFrom::Start(p))?;
        Ok(())
    }
}

pub struct WzImgIter<'r, R> {
    r: &'r mut WzReader<R>,
    q: VecDeque<(Arc<String>, WzDirNode)>,
}

impl<'r, R: WzIO> WzImgIter<'r, R> {
    fn handle_dir(
        &mut self,
        root_name: &str,
        dir: &WzDirHeader,
    ) -> anyhow::Result<(Arc<String>, WzDir)> {
        dbg!(dir);
        let node = dbg!(self.r.read_dir_node(dir)?);
        let node_name = Arc::new(format!("{}/{}", root_name, dir.name.as_str().unwrap()));

        self.q.extend(
            node.entries
                .0
                .iter()
                .map(|x| (node_name.clone(), x.clone())),
        );

        Ok((node_name.clone(), node))
    }
}

impl<'r, R> Iterator for WzImgIter<'r, R>
where
    R: WzIO,
{
    type Item = anyhow::Result<(String, WzImgHeader)>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((root_name, node)) = self.q.pop_front() {
            match node {
                WzDirNode::Dir(dir) => {
                    if let Err(err) = self.handle_dir(root_name.as_str(), &dir) {
                        return Some(Err(err));
                    }
                }
                WzDirNode::Img(img) => {
                    let name = format!("{}/{}", root_name, img.name.as_str().unwrap());
                    return Some(Ok((name, img)));
                }
                _ => {
                    continue;
                }
            }
        }

        None
    }
}

#[cfg(feature = "mmap")]
mod mmap {
    use std::{fs::File, io::Cursor, path::Path};

    use memmap2::Mmap;

    use crate::{
        version::{WzRegion, WzVersion},
        WzReader,
    };

    pub type WzReaderMmap = WzReader<Cursor<Mmap>>;

    impl WzReaderMmap {
        pub fn open_file_mmap(
            path: impl AsRef<Path>,
            region: WzRegion,
            version: WzVersion,
        ) -> anyhow::Result<Self> {
            let file = File::open(path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            Self::new(mmap, region, version)
        }

        pub fn new(mmap: Mmap, region: WzRegion, version: WzVersion) -> anyhow::Result<Self> {
            Self::open(Cursor::new(mmap), region, version)
        }
    }
}
