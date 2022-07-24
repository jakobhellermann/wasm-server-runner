use crate::Result;
use std::path::Path;
use tracing::debug;

pub struct WasmBindgenOutput {
    pub js: String,
    pub wasm: Vec<u8>,
}
pub fn generate(wasm_file: &Path) -> Result<WasmBindgenOutput> {
    debug!("running wasm-bindgen...");
    let start = std::time::Instant::now();
    let mut output = wasm_bindgen_cli_support::Bindgen::new()
        .input_path(wasm_file)
        .web(true)?
        .typescript(false)
        .generate_output()?;
    debug!("finished wasm-bindgen (took {:?})", start.elapsed());

    let js = output.js().to_owned();

    debug!("emitting wasm...");
    let start = std::time::Instant::now();
    let wasm = output.wasm_mut().emit_wasm();
    debug!("emitting wasm took {:?}", start.elapsed());

    Ok(WasmBindgenOutput { js, wasm })
}
