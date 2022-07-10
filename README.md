# wasm-server-runner

Allows you to run programs in the browser using web assembly using a simple `cargo run`.

## Usage

### Step 1.

```sh
cargo install wasm-server-runner
```

### Step 2.

Add this to your `~/.cargo/config.toml` (**not** the `Cargo.toml` of your project!):

```toml
[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

### Step 3.

Run programs in the browser using
```sh
cargo run --target wasm32-unknown-unknown
cargo run --target wasm32-unknown-unknown --example example

wasm-server-runner path/to/file.wasm
```

Example output:
```yaml
INFO wasm_server_runner: wasm output is 49.79kb large
INFO wasm_server_runner::server: starting webserver at http://127.0.0.1:1334
```

The website will reload when the server is restarted and serve files relative to the current directory.
