use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};

use dioxus::{html::text, prelude::*};
use id_tree::NodeId;
use image::RgbaImage;
use shroom_wz::{
    l0::{tree::WzTree, WzDirNode, WzImgHeader},
    l1::{
        canvas::WzCanvas,
        tree::{WzImgNode, WzImgTree},
    },
    tree::WzNode,
    version::{WzRegion, WzVersion},
};

use crate::tree::{TreeData, TreeView};

use crate::image_view::ImageView;

//pub type WzFile = re_wz::file::WzReaderMmap;
pub type WzFile = Cursor<Vec<u8>>;
pub type WzFileReader = shroom_wz::file::WzReader<WzFile>;

impl TreeData for WzDirNode {
    fn get_label(&self) -> String {
        match self {
            WzDirNode::Dir(dir) => format!("📁 {}", dir.name.as_str().unwrap()),
            WzDirNode::Nil(_) => todo!(),
            WzDirNode::Link(link) => format!("🔗 {:?}", link),
            WzDirNode::Img(img) => format!("💾 {}", img.name.as_str().unwrap()),
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

    pub fn from_file(filename: &str, file: WzFile, version: WzVersion) -> anyhow::Result<Self> {
        let mut file = shroom_wz::WzReader::open(file, WzRegion::GMS, version)?;
        let tree = WzTree::read(&mut file, Some(filename))?;
        Ok(Self {
            tree,
            reader: RefCell::new(file),
            cached: RefCell::new(HashMap::new()),
        })
    }

    fn load_tree(&self, img: &WzImgHeader) -> anyhow::Result<&WzImgTree> {
        let name = img.name.as_str();
        let tree = self
            .cached
            .borrow_mut()
            .entry(img.offset.0)
            .or_insert_with(|| {
                Rc::new(
                    WzImgTree::read(&mut self.reader.borrow_mut().img_reader(img).unwrap(), name)
                        .unwrap(),
                )
            })
            .clone();

        // Safety: The cache holds the RC alive until
        // It's dropped from the cache
        // Since the cache is never dropped It means It lives as long as &self does
        Ok(unsafe { std::mem::transmute(tree.as_ref()) })
    }

    fn load_canvas(&self, img: &WzImgHeader, canvas: &WzCanvas) -> anyhow::Result<RgbaImage> {
        self.reader
            .borrow_mut()
            .img_reader(img)?
            .read_canvas(canvas)?
            .to_rgba_image()
    }
}

pub enum WzContentData {
    Image(Rc<RgbaImage>),
    Text(String),
    None,
}

#[inline_props]
fn WzContentView(cx: Scope, content: UseState<WzContentData>) -> Element {
    cx.render(match content.get() {
        WzContentData::Image(img) => rsx!(div {
            ImageView {
                image: img.clone()
            }
        }),
        WzContentData::Text(ref txt) => rsx!(div {
            div {
                class: "card",
                div {
                    class: "card-body",
                    txt.clone()
                }
            }
        }),
        _ => rsx!(div {
            div {
                class: "card",
                div {
                    class: "card-body"
                }
            }
        }),
    })
}

#[inline_props]
fn WzImgView<'wz>(
    cx: Scope<'wz>,
    wz: &'wz WzData,
    img: &'wz WzImgHeader,
    on_select: EventHandler<'wz, &'wz WzImgNode>,
) -> Element {
    let img_tree = wz.load_tree(img).expect("Must load img");
    let tree = img_tree.get_tree();

    cx.render(rsx! {
        TreeView {
            data: tree,
            on_select: move |node: NodeId| on_select.call(tree.get(&node).unwrap().data()),
        }
    })
}

#[inline_props]
fn WzView<'wz>(cx: Scope<'wz>, wz: &'wz WzData) -> Element {
    let tree = wz.tree.get_tree();

    let selected_img_node = use_state::<Option<NodeId>>(cx, || None);
    let content = use_state(cx, || WzContentData::None);

    let selected_img = use_memo(cx, (selected_img_node.get(),), move |(node,)| {
        let Some(node) = node else {
            return None;
        };

        let img_data = wz.tree.get_tree().get(&node).unwrap().data();
        let WzDirNode::Img(img) = img_data else {
            return None;
        };

        Some(img.clone())
    });

    let on_select_node = |node: &'wz WzImgNode| {
        content.set(if let Some(ref canvas) = node.canvas {
            let img = wz
                .load_canvas(selected_img.as_ref().unwrap(), canvas)
                .unwrap();
            WzContentData::Image(Rc::new(img))
        } else {
            WzContentData::None
        });
    };

    let img_view = selected_img.as_ref().map(move |img| {
        rsx!(div {
                class: "col-md-4 overflow-auto vh-100",
                WzImgView {
                    wz: wz,
                    img: img,
                    on_select: on_select_node
                }
        })
    });

    cx.render(rsx! {
        div {
            class: "row justify-content-start",
            div {
                class: "col col-md-4 overflow-auto vh-100",
                TreeView {
                    data: tree,
                    on_select: move |node: NodeId| selected_img_node.set(Some(node))
                }
            }
            img_view

            div {
                class: "col col-md-4 overflow-auto vh-100",
                WzContentView {
                    content: content.clone()
                }
            }
        }
    })
}

#[inline_props]
pub fn WzApp(cx: Scope, wz: UseState<Option<Rc<WzData>>>) -> Element {
    cx.render(rsx! {
        WzView {
            wz: wz.get().as_ref().unwrap()
        }
    })
}
