# md-edit [WIP]

WYSIWYG markdown text editing block

to increase performance only the text currently in view gets rendered

---

requires rust nightly

```sh
# build and develop for desktop
cargo tauri dev

# build and release for desktop
cargo tauri build
```

## prerequisites

```sh
# tauri CLI
cargo install --locked tauri-cli

# Rust nightly (required by Leptos)
rustup toolchain install nightly --allow-downgrade

# WASM target
rustup target add wasm32-unknown-unknown

# trunk WASM bundler
cargo install --locked trunk

# `wasm-bindgen` for Apple M1 chips (required by Trunk)
cargo install --locked wasm-bindgen-cli

# `esbuild` as dependency of `tauri-sys` crate (used in UI)
npm install --global --save-exact esbuild
```
