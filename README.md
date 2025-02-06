# rust-wasm-touch
A simple WebAssembly module in Rust draws a bunch of triangles. Try to use GPU-based rendering.

You have to have pre-installed `cargo install wasm-bindgen-cli`.

To compile wasm do
```
wasm-pack build --target web
```

Use **miniserve** which is installable via Cargo
```
cargo install miniserve
miniserve . --index "index.html" -p 8080
```

Open http://0.0.0.0:8080/ on your web-browser.