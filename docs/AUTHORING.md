# Module authoring

## Quick start

1. Copy the closest template from `templates/` into a new public GitHub repository.
2. Choose a globally unique reverse-DNS module ID. Community modules cannot use `rayslash.*` or author `rayslash`.
3. Complete `module.toml`, `README.md`, `LICENSE`, and the local icon.
4. Validate locally:

   ```sh
   cargo run -p rayslash-module-tool -- validate /path/to/module
   ```

5. Package locally:

   ```sh
   cargo run -p rayslash-module-tool -- package /path/to/module
   ```

6. Publish the `.tar.zst` and `.sha256` files as immutable GitHub Release assets.
7. Submit the repository through a pull request to `rslauncher/rayslash-registry`.

Declarative modules contain data only. WASM modules implement `api/wit/rayslash-module.wit` and receive no WASI filesystem, process, or network API. Network/cache operations go through declared host capabilities.

## Compatibility

`api_version` is a semantic-version requirement for the module API, not the launcher version. API 1.x keeps WIT and manifest behavior backward compatible. A breaking contract change creates API 2.

## Permissions

Declare only what the module needs. Network entries are exact HTTPS origins. Permission expansion during an update requires user confirmation. Command execution is high risk, typed, explicit-activation-only, and never evaluated by a shell.

## Stable IDs and results

Module IDs never change after publication. Provider IDs are stable within a module. Result IDs must identify the same logical item across queries so local learned ranking remains useful.

