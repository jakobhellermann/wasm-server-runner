use std::path::PathBuf;

use anyhow::{anyhow, ensure};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::server::Options;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

mod server;
mod wasm_bindgen;

fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug,walrus=error"));
    tracing_subscriber::fmt::fmt().without_time().with_env_filter(filter).init();

    let title = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "".to_string());

    let options = Options { title };

    let wasm_file = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("expected wasm file as argument"))?;

    let is_wasm_file = wasm_file.extension().map_or(false, |e| e == "wasm");
    ensure!(is_wasm_file, "expected to be run with a wasm target");

    let output = wasm_bindgen::generate(&wasm_file)?;

    info!("wasm output is {} large", pretty_size(output.compressed_wasm.len()));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(server::run_server(options, output))?;

    Ok(())
}

fn pretty_size(size_in_bytes: usize) -> String {
    let size_in_kb = size_in_bytes as f32 / 1024.0;
    if size_in_kb < 1024.0 {
        return format!("{:.2}kb", size_in_kb);
    }

    let size_in_mb = size_in_kb / 1024.0;
    return format!("{:.2}mb", size_in_mb);
}
