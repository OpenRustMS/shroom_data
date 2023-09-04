#![allow(non_snake_case)]

pub mod tree;
pub mod web_map;

use anyhow::anyhow;
use js_sys::Uint8Array;
use wasm_bindgen::{Clamped, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};

use dioxus::prelude::*;
use id_tree::NodeId;
use image::RgbaImage;
use shroom_wz::{
    l0::{tree::WzTree, WzDirNode, WzImgHeader},
    l1::tree::{WzImgNode, WzImgTree},
    version::WzRegion,
};

use tree::{Tree, TreeData};

//pub type WzFile = re_wz::file::WzReaderMmap;
pub type WzFile = Cursor<Vec<u8>>;
pub type WzFileReader = shroom_wz::file::WzReader<WzFile>;

impl TreeData for WzDirNode {
    fn get_label(&self) -> String {
        match self {
            WzDirNode::Dir(dir) => format!("[DIR]{}", dir.name.as_str().unwrap()),
            WzDirNode::Nil(_) => todo!(),
            WzDirNode::Link(link) => format!("[LINK]{:?}", link),
            WzDirNode::Img(img) => format!("[IMG]{}", img.name.as_str().unwrap()),
        }
    }

    fn can_select(&self) -> bool {
        matches!(self, WzDirNode::Img(_))
    }
}

impl TreeData for WzImgNode {
    fn get_label(&self) -> String {
        self.name.clone()
    }

    fn can_select(&self) -> bool {
        self.canvas.is_some()
    }
}

pub struct WzData {
    tree: WzTree,
    reader: RefCell<WzFileReader>,
    cached: RefCell<HashMap<u32, Rc<WzImgTree>>>,
}

impl WzData {
    #[cfg(feature = "mmap")]
    fn load(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut file = re_wz::WzReader::open_file_mmap(file, WzRegion::GMS, 95.into())?;
        Self::from_file(file)
    }

    fn from_file(file: WzFile, version: usize) -> anyhow::Result<Self> {
        let mut file = shroom_wz::WzReader::open(file, WzRegion::GMS, version.into())?;
        let tree = WzTree::read(&mut file)?;
        Ok(Self {
            tree,
            reader: RefCell::new(file),
            cached: RefCell::new(HashMap::new()),
        })
    }

    fn load_tree(&self, img: &WzImgHeader) -> anyhow::Result<Rc<WzImgTree>> {
        log::info!("Loading tree...");
        let tree = self
            .cached
            .borrow_mut()
            .entry(img.offset.0)
            .or_insert_with(|| {
                Rc::new(
                    WzImgTree::read(&mut self.reader.borrow_mut().img_reader(img).unwrap())
                        .unwrap(),
                )
            })
            .clone();
        Ok(tree)
    }
}
fn image_to_imgdata(img: &RgbaImage) -> web_sys::ImageData {
    let data = Clamped(img.as_raw().as_slice());
    web_sys::ImageData::new_with_u8_clamped_array_and_sh(data, img.width(), img.height())
        .expect("Img data")
}

#[inline_props]
fn Image(cx: Scope, image: UseRef<Option<RgbaImage>>) -> Element {
    let canvas_ref = use_ref::<Option<HtmlCanvasElement>>(cx, || None);

    use_effect(cx, (canvas_ref, image), |(canvas_ref, image)| async move {
        let canv_r = canvas_ref.read();

        let Some(canvas) = canv_r.as_ref() else {
            return;
        };

        log::info!("Loading image...");

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let data = image_to_imgdata(image.read().as_ref().unwrap());
        log::info!("Loaded canvas");
        ctx.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        ctx.put_image_data(&data, 0., 0.).unwrap();
    });

    cx.render(rsx! {
        canvas {
            id: "re-img-canvas",
            onmounted: |ev| {
                let el = ev.get_raw_element().expect("Must access element");
                log::info!("el: {el:?} {:?}", el.type_id());
                let el = el.downcast_ref::<web_sys::Element>().expect("Must be element");
                let canvas = el.dyn_ref::<HtmlCanvasElement>().expect("Must be canvas");
                *canvas_ref.write() = Some(canvas.clone());
            }
        }
    })

    /*
    cx.render(rsx! {
        img {
            src: "{data}"
        }
    })*/
}

#[inline_props]
fn WzImgView<'a>(cx: Scope<'a>, data: &'a WzData, img: &'a WzImgHeader) -> Element {
    let img_tree = data.load_tree(img).expect("Must load img");
    let img_canvas = use_ref(cx, || None);

    //TODO find a way to get rid of that unsafe
    let tree: &'a id_tree::Tree<WzImgNode> = unsafe { std::mem::transmute(img_tree.get_tree()) };

    cx.render(rsx! {
        div {
            class: "col overflow-auto vh-100",
            Tree {
                data: tree,
                on_select: move |node: NodeId| {
                    if let Some(ref canvas) = tree.get(&node).unwrap().data().canvas {
                        let canvas_data = data.reader.borrow_mut().img_reader(img).unwrap().read_canvas(canvas).unwrap();
                        let img_data = canvas_data.to_rgba_image().expect("Img");
                        img_canvas.set(Some(img_data));
                    } else {
                        img_canvas.set(None);
                    }
                }
            } 
        }
        if img_canvas.read().is_some() {
            Some(rsx!(div {
                class: "col overflow-auto vh-100",
                Image {
                    image: img_canvas.clone()
                }
            }))
        }
    })
}

#[inline_props]
fn WzView<'a>(cx: Scope<'a>, data: &'a WzData) -> Element {
    let tree = data.tree.get_tree();
    let selected_node = use_state::<Option<NodeId>>(cx, || None);
    let img_view = selected_node.as_ref().map(|node| {
        let img_data = data.tree.get_tree().get(node).unwrap().data();
        let WzDirNode::Img(img) = img_data else {
            todo!("Not img");
        };

        rsx!(WzImgView {
            data: data,
            img: img
        })
    });

    cx.render(rsx! {
        div {
            class: "row",
            div {
                class: "col overflow-auto vh-100",
                Tree {
                    data: tree,
                    on_select: move |node: NodeId| selected_node.set(Some(node))
                }
            }
            img_view
        }
    })
}

/*
#[inline_props]
fn App(cx: Scope, file: PathBuf) -> Element {
    let data = use_state(cx, || WzData::load(file).expect("Must load"));
    cx.render(rsx! {
        div {
            class: "container p-1",
            WzView {
                data: data
            }
        }
    })
}*/

/*
static CUSTOM_HEAD: &str = r#"
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-GLhlTQ8iRABdZLl6O3oVMWSktQOp6b7In1Zl3/Jr59b6EGGoI1aFkw7cmDA6j6gD" crossorigin="anonymous">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.3.0/font/bootstrap-icons.css">
"#;

fn launch_desktop() -> anyhow::Result<()> {
    hot_reload_init!(Config::new().with_rebuild_command("cargo run"));

    let file = rfd::FileDialog::new()
        .add_filter("wz file", &["wz"])
        .set_directory("/")
        .pick_file()
        .expect("Must select file");

    dbg!(&file);

    dioxus_desktop::launch_with_props(
        App,
        AppProps {
            file
        },
        dioxus_desktop::Config::default().with_custom_head(CUSTOM_HEAD.to_string())
    );
    Ok(())
}*/

fn WebApp(cx: Scope) -> Element {
    let data = use_state::<Option<WzData>>(cx, || None);

    async fn read_first_file(input_id: &str, version: usize) -> anyhow::Result<WzData> {
        let win = web_sys::window()
            .unwrap()
            .document()
            .expect("should have a document.");
        let el = win
            .get_element_by_id(input_id)
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let files = el.files().unwrap();
        let file = files.get(0).expect("should contain one file");

        let buf = JsFuture::from(file.array_buffer())
            .await
            .map_err(|_| anyhow!("Unable to load file"))?;
        let buf = buf.dyn_into::<js_sys::ArrayBuffer>().expect("array buffer");
        let array = Uint8Array::new(&buf);
        let bytes: Vec<u8> = array.to_vec();

        console_error_panic_hook::set_once();

        let data = WzData::from_file(Cursor::new(bytes), version)?;

        Ok(data)
    }

    let r_data = data.as_ref();
    let main = if let Some(wz_data) = r_data {
        rsx! {
            div {
                class: "container p-3",
                WzView {
                    data: wz_data
                }
            }
        }
    } else {
        rsx! {
            form {
                class: "p-3",
                //prevent_default: "onsubmit",
                onsubmit: move |ev: Event<FormData>| {
                    // Try to load the file

                    let selected_version = ev.values.get("version").and_then(|v| v.first()).expect("must have version");
                    let version: usize = selected_version.parse().expect("Version must be a number");

                    let files = &ev.inner().files.is_some();
                    log::info!("Event: {ev:?} -- {files}");
                    cx.spawn({
                        let data = data.to_owned();
                        async move {
                            let wz_data = read_first_file("filegetter", version).await.unwrap();
                            data.set(Some(wz_data));
                        }
                    });
                },
                div {
                    class: "mb-3",
                    input {
                        id: "filegetter",
                        r#type: "file",
                        class: "form-control",
                        name: "file"
                    }
                },
                div {
                    class: "mb-3",
                    label {
                        class: "form-label",
                        "Version"
                    }
                    input {
                        r#type: "number",
                        class: "form-control",
                        name: "version",
                        value: "95",
                    }
                },
                input {
                    class: "btn btn-primary",
                    r#type: "submit",
                    "Load"
                }
            }
        }
    };

    /*

        static CUSTOM_HEAD: &str = r#"
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-GLhlTQ8iRABdZLl6O3oVMWSktQOp6b7In1Zl3/Jr59b6EGGoI1aFkw7cmDA6j6gD" crossorigin="anonymous">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.3.0/font/bootstrap-icons.css">
    "#;
     */

    cx.render(rsx!{
        head {
            title {
                "reWZ"
            }
            link {
                href: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css",
                rel: "stylesheet"
            }
            link {
                href: "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.3.0/font/bootstrap-icons.css",
                rel: "stylesheet"
            }
        }
        body {
            main
        }
    })
}

fn launch_web() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    dioxus_web::launch(WebApp);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    launch_web()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn load_skill() -> anyhow::Result<()> {
        let mut f = File::open("../../game_data/wz/Item.wz")?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let _rdr = WzData::from_file(Cursor::new(buf), 95)?;

        Ok(())
    }
}
