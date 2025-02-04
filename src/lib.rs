use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub fn draw_circle() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let canvas = document
        .get_element_by_id("myCanvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    context.begin_path();
    context.arc(50.0, 50.0, 40.0, 0.0, std::f64::consts::PI * 2.0)?;
    context.set_fill_style_str("blue");
    context.fill();

    Ok(())
}
