# wasm-server-runner

Allows you to run programs in the browser using web assembly using a simple `cargo run`.

## Usage

### Step 1.

```sh
rustup target add wasm32-unknown-unknown
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
INFO wasm_server_runner: starting webserver at http://127.0.0.1:1334
```

The website will reload when the server is restarted and serve files relative to the current directory.

## Configuration options

All configuration options can be specified via environment variables.

<details>
<summary>WASM_SERVER_RUNNER_ADDRESS</summary>

Default: `127.0.0.1`
Control the address that the server listens on. Set to `0.0.0.0` to allow access from anywhere.

</details>

<details>
<summary>WASM_SERVER_RUNNER_DIRECTORY</summary>

Default: `.`

Can be used to specify where relative path requests are loaded from.

</details>

<details>
<summary>WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML</summary>

Default: _none_

When set, will try to load the custom `index.html` from that path instead of [the
default](static/index.html). Can be relative to the directory.

*Note*: To support both module and non-module style the `index.html` needs to have specific string that will be replaced at runtime. The body should contain something like
```html
{{ NO_MODULE }}
<script type="module">
    // {{ MODULE }}
</script>
```

</details>

<details>
<summary>WASM_SERVER_RUNNER_HTTPS</summary>

Default: `false`

Controls whether https is used.

</details>

<details>
<summary>WASM_SERVER_RUNNER_NO_MODULE</summary>

Default: `false`

Controls whether the wasm-bindgen output uses `module`s or not.
</details>
