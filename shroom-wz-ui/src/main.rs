#![allow(non_snake_case)]

pub mod image_view;
pub mod tree;
pub mod web_map;
pub mod wz;

use std::{io::Cursor, rc::Rc};

use anyhow::anyhow;
use dioxus::prelude::*;
use js_sys::Uint8Array;
use shroom_wz::version::WzVersion;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlInputElement;

use crate::wz::{WzApp, WzData};

async fn read_wz_data(file_input_id: &str, version: WzVersion) -> anyhow::Result<WzData> {
    let win = web_sys::window()
        .unwrap()
        .document()
        .expect("should have a document.");
    let el = win
        .get_element_by_id(file_input_id)
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let files = el.files().unwrap();
    let file = files.get(0).expect("should contain one file");
    let filename = file.name();

    let buf = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| anyhow!("Unable to load file"))?;
    let buf = buf.dyn_into::<js_sys::ArrayBuffer>().expect("array buffer");
    let array = Uint8Array::new(&buf);
    let bytes: Vec<u8> = array.to_vec();

    console_error_panic_hook::set_once();

    WzData::from_file(&filename, Cursor::new(bytes), version)
}

fn parse_version(form_data: &FormData) -> anyhow::Result<WzVersion> {
    let version = form_data
        .values
        .get("version")
        .and_then(|v| v.first())
        .ok_or(anyhow!("Must have version"))?;
    let version: usize = version.parse().map_err(|_| anyhow!("Invalid version"))?;
    Ok(version.into())
}

#[inline_props]
fn FileForm(cx: Scope, wz: UseState<Option<Rc<WzData>>>) -> Element {
    let alert_error = use_state(cx, || None);

    let load_file = |version: WzVersion| {
        to_owned!(wz);
        to_owned![alert_error];
        cx.spawn({
            async move {
                match read_wz_data("filegetter", version).await {
                    Ok(wz_data) => {
                        wz.set(Some(Rc::new(wz_data)));
                        return;
                    }
                    Err(err) => {
                        alert_error.set(Some(format!("{:?}", err)));
                    }
                }
            }
        });
    };

    cx.render(rsx! {
        div {
            class: "row justify-content-center",
            div  {
                class: "col-md-6",
        if let Some(msg) = alert_error.get() {
            Some(rsx!(div {
                class: "alert alert-danger",
                "{msg}"
            }))
        }
        form {
            class: "p-3",
            //prevent_default: "onsubmit",
            onsubmit: move |ev: Event<FormData>| {
                let Ok(selected_version) = parse_version(&ev.data) else {
                        alert_error.set(Some("Invalid version".to_string()));
                        return;
                    };
                load_file(selected_version);
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
    }}
    })
}

/// Convience function
fn WebApp(cx: Scope) -> Element {
    let wz = use_state::<Option<Rc<WzData>>>(cx, || None);

    let main = if wz.get().is_some() {
        rsx!(WzApp { wz: wz.clone() })
    } else {
        rsx!(FileForm { wz: wz.clone() })
    };

    cx.render(rsx! {
        div {
            class: "container-fluid",
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
