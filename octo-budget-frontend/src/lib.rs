#![cfg(target_arch = "wasm32")]

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

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub async fn greet2() -> Result<String, String> {
    use models::SerializedBudget;
    let r = reqwest::Client::new()
        .get("https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master")
        .send()
        .await.expect("failed to send request");

    let s = format!("response: {:?}", r);
    log(&s);

    let x = SerializedBudget::default();
    let s = format!("budget is: {}", serde_json::to_string(&x).unwrap());
    alert(&s);

    Ok(String::new())
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

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn simple_addition() {
        assert_eq!("123.00", calc("122 + 1").unwrap());
    }

    #[wasm_bindgen_test]
    fn more_complex() {
        assert_eq!("123.00", calc("(60 * 2 + 2) + 1").unwrap());
    }

    #[wasm_bindgen_test]
    fn with_floats() {
        assert_eq!("123.00", calc("122.5 + 0.5").unwrap());
    }

    #[wasm_bindgen_test]
    fn with_floats_devided_by_comma() {
        assert_eq!("123.00", calc("122,5 + 0,5").unwrap());
    }

    #[wasm_bindgen_test]
    fn add_5_percent() {
        assert_eq!("105.00", add_percent("100", 5).unwrap());
    }

    #[wasm_bindgen_test]
    fn add_12_percent() {
        assert_eq!("112.00", add_percent("50 + 50", 12).unwrap());
    }
}
