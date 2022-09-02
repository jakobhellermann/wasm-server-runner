use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(inline_js = "export function add(a, b) { return a + b; }")]
extern "C" {
    fn add(a: u32, b: u32) -> u32;
}
#[wasm_bindgen(inline_js = "export function add2(a, b) { return a + b; }")]
extern "C" {
    fn add2(a: u32, b: u32) -> u32;
}
#[wasm_bindgen(module = "/examples/snippets/snippet.js")]
extern "C" {
    fn add3(a: u32, b: u32) -> u32;
}

fn main() {
    log(&add(2, 3).to_string());
    log(&add2(2, 3).to_string());
    log(&add3(2, 3).to_string());
}
