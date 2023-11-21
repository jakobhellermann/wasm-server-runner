use crate::server::Options;
use crate::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

pub struct WasmBindgenOutput {
    pub js: String,
    pub wasm: Vec<u8>,
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

    Ok(WasmBindgenOutput { js, wasm, snippets, local_modules })
}
