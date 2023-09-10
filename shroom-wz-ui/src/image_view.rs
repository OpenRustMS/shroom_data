use std::rc::Rc;

use dioxus::prelude::*;
use image::RgbaImage;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn image_to_imgdata(img: &RgbaImage) -> web_sys::ImageData {
    let data = Clamped(img.as_raw().as_slice());
    web_sys::ImageData::new_with_u8_clamped_array_and_sh(data, img.width(), img.height())
        .expect("Img data")
}

#[inline_props]
pub fn ImageView(cx: Scope, image: Rc<RgbaImage>) -> Element {
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

        let data = image_to_imgdata(&image);
        log::info!("Loaded canvas");
        ctx.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
        ctx.put_image_data(&data, 0., 0.).unwrap();
    });

    cx.render(rsx! {
        canvas {
            onmounted: |ev| {
                let canvas = ev.get_raw_element().expect("Must access element")
                    .downcast_ref::<web_sys::Element>().expect("Must be element")
                    .dyn_ref::<HtmlCanvasElement>().expect("Must be canvas");
                *canvas_ref.write() = Some(canvas.clone());
            }
        }
    })
}
