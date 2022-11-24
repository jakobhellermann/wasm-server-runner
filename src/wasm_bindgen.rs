use crate::server::Options;
use crate::Result;
use anyhow::Context;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

const COMPRESSION_LEVEL: u32 = 2;

pub struct WasmBindgenOutput {
    pub js: String,
    pub br_compressed_wasm: Vec<u8>,
    pub gzip_compressed_wasm: Vec<u8>,
    pub snippets: HashMap<String, Vec<String>>,
    pub local_modules: HashMap<String, String>,
}
pub fn generate(options: &Options, wasm_file: &Path) -> Result<WasmBindgenOutput> {
    debug!("running wasm-bindgen...");
    let start = std::time::Instant::now();
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen.input_path(wasm_file).typescript(false).weak_refs(true).reference_types(true);

    if options.no_module {
        bindgen.no_modules(true)?;
    } else {
        bindgen.web(true)?;
    }

    let mut output = bindgen.generate_output()?;
    debug!("finished wasm-bindgen (took {:?})", start.elapsed());

    let js = output.js().to_owned();
    let snippets = output.snippets().clone();
    let local_modules = output.local_modules().clone();

    debug!("emitting wasm...");
    let start = std::time::Instant::now();
    let wasm = output.wasm_mut().emit_wasm();
    debug!("emitting wasm took {:?}", start.elapsed());

    debug!("br compressing wasm...");
    let start = std::time::Instant::now();
    let br_compressed_wasm = br_compress(&wasm).context("failed to compress wasm file")?;
    debug!("compressing took {:?}", start.elapsed());

    debug!("gzip compressing wasm...");
    let start = std::time::Instant::now();
    let gzip_compressed_wasm = gzip_compress(&wasm).context("failed to compress wasm file")?;
    debug!("compressing took {:?}", start.elapsed());

    Ok(WasmBindgenOutput { js, br_compressed_wasm, gzip_compressed_wasm, snippets, local_modules })
}

fn br_compress(mut bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use brotli::enc::{self, BrotliEncoderParams};

    let mut output = Vec::new();
    enc::BrotliCompress(
        &mut bytes,
        &mut output,
        &BrotliEncoderParams {
            quality: 5, // https://github.com/jakobhellermann/wasm-server-runner/pull/22#issuecomment-1235804905
            ..Default::default()
        },
    )?;

    Ok(output)
}

fn gzip_compress(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(COMPRESSION_LEVEL));

    encoder.write_all(bytes)?;
    let compressed = encoder.finish()?;

    Ok(compressed)
}
