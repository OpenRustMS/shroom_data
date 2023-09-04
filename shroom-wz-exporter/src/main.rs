use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
};

use clap::Parser;
use serde_json::{Map, Number};
use shroom_wz::{
    canvas,
    file::{WzIO, WzImgReader},
    l1::{
        obj::WzObject,
        prop::{WzObj, WzProperty, WzValue, WzVector2D},
    },
    version::WzVersion,
};

#[derive(Parser, Debug)]
#[command(name = "shroom-wz-exporter")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(short, long)]
    out_dir: Option<PathBuf>,
}

fn wz_to_json(val: &WzValue) -> anyhow::Result<serde_json::Value> {
    Ok(match val {
        WzValue::Null => serde_json::Value::Null,
        WzValue::Short1(v) | WzValue::Short2(v) => (*v as i32).into(),
        WzValue::Int1(v) | WzValue::Int2(v) => v.0.into(),
        WzValue::Long(v) => v.0.into(),
        WzValue::F32(v) => Number::from_f64(v.0 as f64).into(),
        WzValue::F64(v) => Number::from_f64(*v).into(),
        WzValue::Str(v) => v.as_str().unwrap_or("invalid").to_string().into(),
        WzValue::Obj(_obj) => serde_json::Value::Object(Map::new()),
    })
}

fn vec2_to_json(vec2: &WzVector2D) -> serde_json::Value {
    let mut map = Map::new();
    map.insert("x".to_string(), vec2.x.0.into());
    map.insert("y".to_string(), vec2.y.0.into());
    serde_json::Value::Object(map)
}

fn handle_img<R: WzIO>(path: PathBuf, mut img_reader: WzImgReader<R>) -> anyhow::Result<()> {
    let root = img_reader.read_root_obj()?;

    let mut root_obj = Map::new();

    let mut q = VecDeque::new();
    q.push_back(("".to_string(), "root".to_string(), root));

    while let Some((path, name, obj)) = q.pop_front() {
        // We insert the data in the generated map
        let mut map = &mut root_obj;
        for part in path.split('/').skip(1) {
            map = map.get_mut(part).unwrap().as_object_mut().unwrap()
        }

        match obj {
            WzObject::Property(prop) => {
                // Create new map
                for entry in prop.entries.0 {
                    let name = entry.name.as_str().unwrap().to_string();
                    if let WzValue::Obj(obj) = entry.val {
                        let object = img_reader.read_obj(&obj)?;
                        let path = format!("{}/{}", path, name);
                        map.insert(name.clone(), Map::new().into());
                        q.push_back((path, name, object));
                        continue;
                    }

                    let val = wz_to_json(&entry.val)?;
                    map.insert(name.clone(), val);
                }
            }
            WzObject::Canvas(canvas) => {
                if let Some(prop) = canvas.other_byte {
                    for entry in prop.entries.0 {
                        let name = entry.name.as_str().unwrap().to_string();
                        if let WzValue::Obj(obj) = entry.val {
                            let object = img_reader.read_obj(&obj)?;
                            let path = format!("{}/{}", path, name);
                            map.insert(name.clone(), Map::new().into());
                            q.push_back((path, name, object));
                            continue;
                        }

                        let val = wz_to_json(&entry.val)?;
                        map.insert(name.clone(), val);
                    }
                }
            }
            WzObject::Convex2D(convex) => {
                map.insert(
                    name,
                    serde_json::Value::Array(convex.0.iter().map(|x| vec2_to_json(x)).collect()),
                );
            }
            WzObject::SoundDX8(_sound) => {}
            WzObject::UOL(_uol) => {}
            WzObject::Vec2(vec2) => {
                map.insert(name, vec2_to_json(&vec2));
            }
            _ => {}
        };
    }

    // Write file

    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = std::fs::File::create(path.with_extension("json"))?;
    serde_json::to_writer_pretty(&mut file, &root_obj)?;

    Ok(())
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
    let out_dir = PathBuf::from("out");
    for (path, img) in imgs.iter() {
        let mut path = path.clone();
        path.remove(0);
        handle_img(out_dir.join(path), file.img_reader(img)?)?;
    }

    Ok(())
}
