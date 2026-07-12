# Hello module template

Replace every `example` identity and repository value before publishing. Keep `module.toml` and `Cargo.toml` versions identical.

## Build and test

Install the free Rust toolchain and `cargo-component`, then generate bindings and build:

```sh
cargo install cargo-component --locked --version 0.21.1
cargo component bindings
cargo fmt --all
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo component build --release --locked --target wasm32-unknown-unknown
```

Copy the component into a clean package directory with `module.toml`, `README.md`, `LICENSE`, and `icon.svg` as `module.wasm`. Validate and package it with a checkout of the SDK:

```sh
mkdir -p dist
cp module.toml README.md LICENSE icon.svg dist/
cp target/wasm32-unknown-unknown/release/rayslash_module_example.wasm dist/module.wasm
cargo run --manifest-path ../rayslash-module-sdk/Cargo.toml -p rayslash-module-tool --locked -- validate dist
cargo run --manifest-path ../rayslash-module-sdk/Cargo.toml -p rayslash-module-tool --locked -- package dist
```

Publish both generated files in an immutable GitHub Release, then follow the SDK release/submission guide. Never replace an existing release asset; publish a new semantic version.

The included workflows repeat these checks for every pull request and create the package assets for every `v*` tag. Update their package/release file name if you rename the Rust crate.
