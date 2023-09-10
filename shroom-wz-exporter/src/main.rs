use std::path::PathBuf;

use clap::Parser;

use jtd_infer::{HintSet, Hints};
use serde_reflection::{Samples, Tracer, TracerConfig};
use shroom_wz::version::WzVersion;
use toml::{Table, Value};

use crate::{schema::Schema, skill::process_skill_value};

pub mod schem;
pub mod schema;
pub mod skill;

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
    //let args = Args::parse();

    let file = std::env::home_dir()
        .unwrap()
        .join("Documents/open-rust-ms/Skill.wz");

    let mut file = shroom_wz::WzReader::open_file_mmap(
        file,
        shroom_wz::version::WzRegion::GMS,
        WzVersion(95),
    )?;

    let imgs = file.img_iter().collect::<anyhow::Result<Vec<_>>>()?;
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
