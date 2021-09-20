use std::path::Path;

use crate::Result;

pub struct WasmBindgenOutput {
    pub js: String,
    pub wasm: Vec<u8>,
}
pub fn generate(wasm_file: &Path) -> Result<WasmBindgenOutput> {
    let mut output = wasm_bindgen_cli_support::Bindgen::new()
        .input_path(wasm_file)
        .web(true)?
        .typescript(false)
        .generate_output()?;

    Ok(WasmBindgenOutput {
        js: output.js().to_owned(),
        wasm: output.wasm_mut().emit_wasm(),
    })
}
