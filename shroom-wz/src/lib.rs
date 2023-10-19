pub mod canvas;
pub mod crypto;
pub mod file;
pub mod keys;
pub mod l0;
pub mod l1;
pub mod ty;
pub mod util;
pub mod val;
pub mod version;

#[cfg(feature = "mmap")]
pub use file::mmap::WzReaderMmap;
pub use file::WzReader;

#[cfg(test)]
mod tests {

    use rodio::{OutputStream, Source};

    use crate::{
        l0::{tree::WzTree, WzDirNode},
        l1::obj::WzObject,
        val::WzValue,
        version::{self, WzVersion},
        WzReader,
    };

    fn get_file_from_home(path: &str) -> std::path::PathBuf {
        #[allow(deprecated)]
        let home = std::env::home_dir().unwrap();
        home.join(path)
    }

    #[test]
    fn load3() -> anyhow::Result<()> {
        let mut skill = WzReader::open_file(
            get_file_from_home("Documents/open-rust-ms/Skill.wz"),
            version::WzRegion::GMS,
            WzVersion(95),
        )?;

        let tree = WzTree::from_reader(&mut skill, None)?;

        dbg!(tree.get_by_path("2212.img").unwrap());

        let link = tree.get_by_path("Dragon/2212.img").unwrap();
        dbg!(&link);
        //dbg!(tree);

        Ok(())
    }

    #[test]
    fn load() -> anyhow::Result<()> {
        let mut item = WzReader::open_file(
            get_file_from_home("Documents/open-rust-ms/Item.wz"),
            version::WzRegion::GMS,
            WzVersion(95),
        )?;

        let root = item.read_root_dir()?;
        let WzDirNode::Dir(ref pet) = root.entries.0[1] else {
            anyhow::bail!("Invalid pet");
        };
        let pets = item.read_dir_node(pet)?;
        let WzDirNode::Img(ref pet0) = pets.entries.0[0] else {
            anyhow::bail!("Invalid pet");
        };

        let mut img = item.img_reader(pet0)?;
        let root = img.read_root_obj()?;

        let WzObject::Canvas(ref canvas) = img.read_path(&root, "info/icon")? else {
            anyhow::bail!("Invalid canvas");
        };

        let icon = img.read_canvas(canvas)?;
        let icon_img = icon.to_rgba_image()?;
        icon_img.save("icon.png")?;

        let v = WzValue::read(&mut img)?;
        dbg!(&v);

        Ok(())
    }

    #[test]
    fn load_audio() {
        let mut sound = WzReader::open_file(
            get_file_from_home("Downloads/Sound.wz"),
            version::WzRegion::GMS,
            WzVersion(95),
        )
        .unwrap();

        let tree = WzTree::from_reader(&mut sound, None).unwrap();
        let mob = tree.get_img_by_path("BgmGL.img").unwrap();

        let mut img = sound.img_reader(&mob).unwrap();
        let val = WzValue::read(&mut img).unwrap();

        let sound = val
            .get_path("Amorianchallenge")
            .unwrap()
            .as_sound()
            .unwrap();

        let sound_data = sound.read_data(&mut img).unwrap();
        let dec = rodio::Decoder::new(std::io::Cursor::new(sound_data)).unwrap();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        stream_handle.play_raw(dec.convert_samples()).unwrap();
        std::thread::sleep(sound.duration());

        /*

        let entry = img.read_root_obj().unwrap();

        let prop = entry.unwrap_property();
        let first_prop = &prop.entries.0[10];
        let first = first_prop.val.clone().unwrap_obj();

        let entry = img.read_obj(&first).unwrap();

        dbg!(entry);
        let prop = entry.unwrap_property();
        let first_prop = &prop.entries.0[0];
        let first = first_prop.val.clone().unwrap_obj();

        let sound = img.read_obj(&first).unwrap();
        let sound = sound.unwrap_sound_dx_8();
        dbg!(sound.data_size());

        let sound_data = img.read_sound(&sound).unwrap();
        let mut dec = rodio::Decoder::new(std::io::Cursor::new(sound_data)).unwrap();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        stream_handle.play_raw(dec.convert_samples());
        std::thread::sleep(std::time::Duration::from_secs(5));*/
    }
}
