#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use octo_budget_frontend::*;
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
