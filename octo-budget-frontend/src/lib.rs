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

#[wasm_bindgen]
pub fn calc(text: &str) -> Option<String> {
    eval(text)
        .ok()
        .and_then(|value| value.as_f64())
        .map(|number| format!("{:.2}", number))
}