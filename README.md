# rayslash module SDK

This repository is the source of truth for rayslash module API v1, manifest validation, deterministic packaging, schemas, templates, and author documentation.

- [Authoring guide](docs/AUTHORING.md)
- [WIT API](api/wit/rayslash-module.wit)
- [Manifest schema](schemas/module.schema.json)

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The public API remains a development preview until the host and conformance suite are released together.

