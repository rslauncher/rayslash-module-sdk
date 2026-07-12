# rayslash module SDK

This repository is the source of truth for rayslash module API v1, manifest validation, deterministic packaging, schemas, templates, and author documentation.

API v1 accepts sandboxed WASM packages. The `declarative` kind is reserved and is not installable until a future API defines its format and runtime behavior.

- [Authoring guide](docs/AUTHORING.md)
- [API v1 reference](docs/API.md)
- [Release and submission guide](docs/RELEASING.md)
- [WIT API](api/wit/rayslash-module.wit)
- [Manifest schema](schemas/module.schema.json)

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

API v1 is stable. Compatible additions remain within v1; breaking WIT, manifest, or behavior changes require API v2.
