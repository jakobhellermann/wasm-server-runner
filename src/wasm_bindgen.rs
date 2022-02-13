use crate::Result;
use anyhow::Context;
use std::path::Path;
use tracing::debug;

const COMPRESSION_LEVEL: u32 = 2;

pub struct WasmBindgenOutput {
    pub js: String,
    pub compressed_wasm: Vec<u8>,
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

    debug!("compressing wasm...");
    let start = std::time::Instant::now();
    let compressed_wasm = compress(&wasm).context("failed to compress wasm file")?;
    debug!("compressing took {:?}", start.elapsed());

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
