use std::path::{Path, PathBuf};

use clap::Parser;

use shroom_wz::{
    file::WzReaderFile,
    val::{ObjectVal, WzValue},
    version::WzVersion,
    WzReaderMmap,
};

use crate::{schema::Schema, skill::process_skill_value};

pub mod data;
pub mod eval;
pub mod mob;
pub mod schem;
pub mod schema;
pub mod skill;
pub mod skill2;

fn load_wz(p: impl AsRef<Path>) -> anyhow::Result<WzReaderMmap> {
    shroom_wz::WzReader::open_file_mmap(p, shroom_wz::version::WzRegion::GMS, WzVersion(95))
}

fn gen_skill() -> anyhow::Result<()> {
    #[allow(deprecated)]
    let file = std::env::home_dir()
        .unwrap()
        .join("Documents/open-rust-ms/Skill.wz");

    let mut file = load_wz(file)?;

    let imgs = file.traverse_images().collect::<anyhow::Result<Vec<_>>>()?;
    let out_dir = PathBuf::from("out/skill");

    let mut schema = Schema::new();

    for (path, img) in imgs.iter().take(1) {
        let path = path.strip_prefix("/root/").unwrap_or(path);
        let path = out_dir.join(path);
        let val = &file.img_reader(img)?.into_serializer(false)?;
        let val = toml::Value::try_from(val)?;

        let mut toml_root = toml::Value::try_from(&val)?;

        if let Some(tbl) = toml_root.get_mut("skill").and_then(|v| v.as_table_mut()) {
            process_skill_value(tbl);

            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = std::fs::File::create(path.with_extension("json"))?;
            serde_json::to_writer_pretty(&mut file, &tbl)?;
            for (k, v) in tbl.iter() {
                println!("skill : {}", k);
                schema.process_dir("Skill", v.as_table().unwrap())?;
            }
        }
    }

    println!("{}", schema.to_code());
    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "shroom-wz-exporter")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(short, long)]
    out_dir: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    #[allow(deprecated)]
    let file = std::env::home_dir()
        .unwrap()
        .join("Documents/open-rust-ms/Skill.wz");

    let mut file = load_wz(file)?;
    let imgs = file.traverse_images().collect::<anyhow::Result<Vec<_>>>()?;

    for img in imgs.iter() {
        let (path, img) = img;

        let mut img_reader = file.img_reader(&img)?;
        let val = WzValue::read(&mut img_reader)?;
        let val = val.as_object().unwrap();
        if val.get("skill").is_none() {
            dbg!(path);
            if path.contains("Dragon") {
                dbg!(&path);
                dbg!(val);
            }
            continue;
        }
        let skills: &ObjectVal = val.must_get_into("skill")?;

        for (k, v) in skills.0.iter() {
            let skill_id = k.parse::<u32>().unwrap();

            let skill_obj = v.as_object().unwrap();
            if skill_obj.get("common").is_none() {
                continue;
            }
            let skill = skill2::Skill::from_value(skill_id, skill_obj).unwrap();
            //dbg!(k);
            if skill.elem_attr.is_some() {
                //dbg!(k);
                //dbg!(&skill);
            }
        }
    }

    //gen_skill()?;

    //let imgs = file.img_iter().collect::<anyhow::Result<Vec<_>>>()?;

    Ok(())
}
