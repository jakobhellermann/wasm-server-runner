use crate::server::Options;
use crate::Result;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use tracing::debug;

pub struct WasmBindgenOutput {
    pub js: String,
    pub wasm: Vec<u8>,
    pub snippets: BTreeMap<String, Vec<String>>,
    pub local_modules: HashMap<String, String>,
}
pub fn generate(options: &Options, wasm_file: &Path) -> Result<WasmBindgenOutput> {
    debug!("running wasm-bindgen...");
    let start = std::time::Instant::now();
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen.input_path(wasm_file).typescript(false);

    if options.no_module {
        bindgen.no_modules(true)?;
    } else {
        bindgen.web(true)?;
    }

    let mut output = match bindgen.generate_output() {
        Ok(output) => output,
        Err(error) => {
            if let Some((wasm_version, runner_version)) =
                extract_error_message_versions(&error.to_string())
            {
                return Err(anyhow::anyhow!(
                    r#"The rust project was linked against a different version of wasm-bindgen.
wasm-server-runner version: {runner_version}
wasm file schema version:   {wasm_version}

To resolve this, update the wasm-bindgen dependency and/or wasm-server-runner binary:
    cargo update -p wasm-bindgen
    cargo install -f wasm-server-runner"#
                ));
            }

            return Err(error);
        }
    };

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

fn extract_error_message_versions(msg: &str) -> Option<(&str, &str)> {
    let (_, msg) = msg.split_once("rust wasm file schema version: ")?;
    let (wasm_schema_version, msg) = msg.split_once("\n     this binary schema version: ")?;
    let (binary_schema_version, _) = msg.split_once('\n')?;

    Some((wasm_schema_version, binary_schema_version))
}
