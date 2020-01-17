extern crate cfg_if;
extern crate wasm_bindgen;
use js_sys::eval;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, octo-budget-frontend!");
}

fn js_eval(text: &str) -> Option<f64> {
    let text = text.replace(",", ".");

    eval(&text).ok().and_then(|value| value.as_f64())
}

#[wasm_bindgen]
pub fn calc(text: &str) -> Option<String> {
    js_eval(text).map(|number| format!("{:.2}", number))
}

#[wasm_bindgen]
pub fn add_percent(text: &str, percent: usize) -> Option<String> {
    js_eval(text)
        .map(|value| value + (value / 100.0 * (percent as f64)))
        .map(|number| format!("{:.2}", number))
}
