## 0.4.0
- add workaround to make audio triggered in wasm play with any user interaction
- make sure body fills whole page
- add ability to use a custom serve directory
- enable `--weak-refs` and `--reference-types` in `wasm-bindgen`

## 0.3.0
- support wasm-bindgen snippets
- use brotli at level 5 for compression (for good speed/size tradeoff)
- support https (also with self-signed certificate)
- add COOP and COEP headers
- add ability to generate non-module JS

## 0.2.4
- keep polling the server for updates forever

## 0.2.2
- allow listening address to be customized with environment variable
- clarify that printed size is compressed

## 0.2.1
- prevent right click on body
- add some more logs
- try to fix fullscreen for bevy apps

## 0.2.0
- pick free port instead of hardcoding `1338`
- poll updates to reload page
- compress wasm file using gzip

## 0.1.0

- initial release
