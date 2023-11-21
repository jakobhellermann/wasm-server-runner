use std::path::PathBuf;

use anyhow::{anyhow, ensure};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::server::Options;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

mod server;
mod wasm_bindgen;

fn bool_option(name: &str, default: bool) -> Result<bool, anyhow::Error> {
    match std::env::var(name) {
        Ok(value) if ["true", "1", "yes"].contains(&value.as_str()) => Ok(true),
        Ok(value) if ["false", "0", "no"].contains(&value.as_str()) => Ok(false),
        Ok(value) => Err(anyhow!("unexpected option {name}={value}, expected true,1 or false,0")),
        Err(_) => Ok(default),
    }
}
fn option(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or(default.to_owned())
}

fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,app=debug,tower_http=debug,walrus=error"));
    tracing_subscriber::fmt::fmt().without_time().with_env_filter(filter).init();

    let title = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "".to_string());
    let address = option("WASM_SERVER_RUNNER_ADDRESS", "127.0.0.1");
    let directory = option("WASM_SERVER_RUNNER_DIRECTORY", ".");
    let https = bool_option("WASM_SERVER_RUNNER_HTTPS", false)?;
    let no_module = bool_option("WASM_SERVER_RUNNER_NO_MODULE", false)?;

    let options = Options { title, address, directory: PathBuf::from(directory), https, no_module };

    let wasm_file = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("expected wasm file as argument"))?;

    let is_wasm_file = wasm_file.extension().map_or(false, |e| e == "wasm");
    ensure!(is_wasm_file, "expected to be run with a wasm target");

    let output = wasm_bindgen::generate(&options, &wasm_file)?;

    info!("uncompressed wasm output is {} in size", pretty_size(output.wasm.len()));

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
    format!("{:.2}mb", size_in_mb)
}
