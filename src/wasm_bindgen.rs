use std::path::Path;

use anyhow::Context;

use crate::Result;

const COMPRESSION_LEVEL: u32 = 2;

pub struct WasmBindgenOutput {
    pub js: String,
    pub compressed_wasm: Vec<u8>,
}
pub fn generate(wasm_file: &Path) -> Result<WasmBindgenOutput> {
    let mut output = wasm_bindgen_cli_support::Bindgen::new()
        .input_path(wasm_file)
        .web(true)?
        .typescript(false)
        .generate_output()?;

    let js = output.js().to_owned();
    let wasm = output.wasm_mut().emit_wasm();

    let compressed_wasm = compress(&wasm).context("failed to compress wasm file")?;

    Ok(WasmBindgenOutput { js, compressed_wasm })
}

fn compress(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(COMPRESSION_LEVEL));

    encoder.write_all(bytes)?;
    let compressed = encoder.finish()?;

    Ok(compressed)
}
