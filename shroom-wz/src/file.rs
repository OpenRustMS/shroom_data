use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use binrw::{BinRead, PosValue};

use crate::{
    canvas::Canvas,
    crypto::WzCrypto,
    l0::{WzDir, WzDirHeader, WzDirNode, WzHeader, WzImgHeader, WzLinkData},
    l1::{
        canvas::WzCanvas,
        obj::WzObject,
        prop::{WzObj, WzPropValue},
        ser::WzImgSerializer,
        sound::WzSound,
    },
    ty::{WzInt, WzOffset},
    util::{BufReadExt, SubReader, WzContext, WzStrTable},
    version::{WzRegion, WzVersion},
};
pub trait WzIO: BufRead + Seek {}
impl<T> WzIO for T where T: BufRead + Seek {}

pub struct WzImgReader<R> {
    r: R,
    crypto: Rc<WzCrypto>,
    str_table: WzStrTable,
}

impl<R> WzImgReader<R>
where
    R: WzIO,
{
    pub fn root_obj(&self) -> WzObj {
        WzObj {
            len: PosValue { val: 0, pos: 0 },
        }
    }

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
                .find(|x| x.name.as_ref().as_str() == part)
                .ok_or_else(|| anyhow::format_err!("Invalid {path}"))?;

            let obj = match &next.val {
                WzPropValue::Obj(ref obj) => obj,
                _ => anyhow::bail!("Invalid obj: {cur:?}"),
            };
            obj_storage = Some(self.read_obj(obj)?);
            cur = obj_storage.as_ref().unwrap();
        }

        obj_storage.ok_or_else(|| anyhow::format_err!("Invalid {path}"))
    }

    /// Read the root object for that image
    pub fn read_root_obj(&mut self) -> anyhow::Result<WzObject> {
        self.r.rewind()?;
        Ok(WzObject::read_le_args(
            &mut self.r,
            WzContext::new(&self.crypto, &self.str_table),
        )?)
    }

    /// Read an object with the given object header
    pub fn read_obj(&mut self, obj: &WzObj) -> anyhow::Result<WzObject> {
        // Check for root
        let ix = if obj.len.pos == 0 && obj.len.val == 0 {
            0
        } else {
            obj.len.pos + 4
        };

        // Skip first index
        self.r.seek(SeekFrom::Start(ix))?;
        Ok(WzObject::read_le_args(
            &mut self.r,
            WzContext::new(&self.crypto, &self.str_table),
        )?)
    }

    fn read_canvas_from<T: BufRead>(mut r: T, canvas: &WzCanvas) -> anyhow::Result<Canvas> {
        let sz = canvas.bitmap_size() as usize;
        let mut img_buf = Vec::with_capacity(sz);
        r.decompress_flate_size(&mut img_buf, sz)?;
        Ok(Canvas::from_data(img_buf, canvas))
    }

    pub fn read_canvas(&mut self, canvas: &WzCanvas) -> anyhow::Result<Canvas> {
        let len = canvas.data_len();
        let off = canvas.data_offset();
        self.r.seek(SeekFrom::Start(off))?;

        let hdr = self.r.peek_u16()?;
        // Match header
        match hdr & 0xFF {
            0x78 => {
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

    pub fn read_sound(&mut self, sound: &WzSound) -> anyhow::Result<Vec<u8>> {
        let offset = sound.offset.pos;
        let ln = sound.data_size();
        let old = self.r.stream_position()?;
        self.r.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0; ln];
        self.r.read_exact(&mut data)?;
        self.r.seek(SeekFrom::Start(old))?;

        Ok(data)
    }

    pub fn into_serializer(self, skip_canvas: bool) -> anyhow::Result<WzImgSerializer<R>> {
        WzImgSerializer::new(self, skip_canvas)
    }
}

#[derive(Debug)]
pub struct WzReader<R> {
    inner: R,
    crypto: Rc<WzCrypto>,
    str_table: WzStrTable,
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
            str_table: WzStrTable::default(),
        })
    }

    pub fn open_img(rdr: R, region: WzRegion, ver: WzVersion) -> anyhow::Result<Self> {
        let data_offset = 0;

        Ok(Self {
            inner: rdr,
            crypto: WzCrypto::from_region(region, ver, data_offset).into(),
            data_offset: data_offset as u64,
            str_table: WzStrTable::default(),
        })
    }

    fn sub_reader(&mut self, offset: u64, size: u64) -> SubReader<'_, R> {
        SubReader::new(&mut self.inner, offset, size)
    }

    pub fn root_offset(&self) -> WzOffset {
        WzOffset(self.data_offset as u32 + 2)
    }

    pub fn read_root_dir(&mut self) -> anyhow::Result<WzDir> {
        // Skip encrypted version at the start
        self.read_dir(self.root_offset().0 as u64)
    }

    pub fn read_dir_node(&mut self, hdr: &WzDirHeader) -> anyhow::Result<WzDir> {
        self.read_dir(hdr.offset.0 as u64)
    }

    fn read_dir(&mut self, offset: u64) -> anyhow::Result<WzDir> {
        self.set_pos(offset)?;
        Ok(WzDir::read_le_args(
            &mut self.inner,
            WzContext::new(&self.crypto, &self.str_table),
        )?)
    }

    pub fn root_img_reader(&mut self) -> io::Result<WzImgReader<SubReader<'_, R>>> {
        // Get size by seeking to end
        let end = self.inner.seek(SeekFrom::End(0))?;
        let off = 0;
        self.set_pos(off)?;
        let crypto = self.crypto.clone();
        let str_table = self.str_table.clone();

        Ok(WzImgReader {
            r: self.sub_reader(off, end),
            crypto,
            str_table,
        })
    }

    pub fn img_reader(&mut self, hdr: &WzImgHeader) -> io::Result<WzImgReader<SubReader<'_, R>>> {
        let off = hdr.offset.into();
        self.set_pos(off)?;
        let crypto = self.crypto.clone();
        let str_table = self.str_table.clone();

        Ok(WzImgReader {
            r: self.sub_reader(off, hdr.blob_size.0 as u64),
            crypto,
            str_table,
        })
    }
    /*
        pub fn link_img_reader(
            &mut self,
            hdr: &WzLinkData,
        ) -> io::Result<WzImgReader<SubReader<'_, R>>> {
            let off = hdr.offset.into();
            self.set_pos(off)?;
            let crypto = self.crypto.clone();
            let str_table = self.str_table.clone();

            Ok(WzImgReader {
                r: self.sub_reader(off, hdr..0 as u64),
                crypto,
                str_table,
            })
        }
    */
    pub fn traverse_images(&mut self) -> WzImgTraverser<'_, R> {
        let mut q = VecDeque::new();
        q.push_back((
            Arc::new("".to_string()),
            WzDirNode::Dir(WzDirHeader::root("root", 1, self.root_offset())),
        ));
        WzImgTraverser { r: self, q }
    }

    pub fn read_path(&mut self, root: &WzDirNode, path: &str) -> anyhow::Result<WzDirNode> {
        let mut cur = root.clone();

        for part in path.split('/') {
            let WzDirNode::Dir(dir) = cur else {
                anyhow::bail!("Invalid dir: {cur:?}");
            };

            let dir = self.read_dir_node(&dir)?;
            let next = dir.get(part).ok_or_else(|| {
                anyhow::format_err!("Invalid {path}: {part} not found in {dir:?}")
            })?;
            cur = next.clone();
        }

        Ok(cur)
    }

    fn set_pos(&mut self, p: u64) -> io::Result<()> {
        self.inner.seek(SeekFrom::Start(p))?;
        Ok(())
    }
}

pub struct WzImgTraverser<'r, R> {
    r: &'r mut WzReader<R>,
    q: VecDeque<(Arc<String>, WzDirNode)>,
}

impl<'r, R: WzIO> WzImgTraverser<'r, R> {
    fn handle_dir(
        &mut self,
        root_name: &str,
        dir: &WzDirHeader,
    ) -> anyhow::Result<(Arc<String>, WzDir)> {
        let node = self.r.read_dir_node(dir)?;
        let node_name = Arc::new(format!("{}/{}", root_name, dir.name.as_str()));

        self.q.extend(
            node.entries
                .0
                .iter()
                .map(|x| (node_name.clone(), x.clone())),
        );

        Ok((node_name.clone(), node))
    }
}

impl<'r, R> Iterator for WzImgTraverser<'r, R>
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
                    let name = format!("{}/{}", root_name, img.name.as_str());
                    return Some(Ok((name, img)));
                }
                WzDirNode::Link(link) => {
                    let img = link.link.link_img;
                    let name = format!("{}/{}", root_name, img.name.as_str());
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
pub mod mmap {
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
