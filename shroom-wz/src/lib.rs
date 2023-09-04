pub mod canvas;
pub mod crypto;
pub mod file;
pub mod keys;
pub mod l0;
pub mod l1;
pub mod tree;
pub mod ty;
pub mod util;
pub mod version;

pub use file::WzReader;

#[cfg(test)]
mod tests {
    use crate::{
        l0::{tree::WzTree, WzDirNode},
        l1::obj::WzObject,
        version::{self, WzVersion},
        WzReader,
    };

    fn get_file_from_home(path: &str) -> std::path::PathBuf {
        #[allow(deprecated)]
        let home = std::env::home_dir().unwrap();
        home.join(path)
    }

    #[test]
    fn load2() -> anyhow::Result<()> {
        let mut skill = WzReader::open_file(
            get_file_from_home("Documents/open-rust-ms/Skill.wz"),
            version::WzRegion::GMS,
            WzVersion(95),
        )?;

        let tree = WzTree::read(&mut skill)?;

        dbg!(tree);

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

        Ok(())
    }
}
