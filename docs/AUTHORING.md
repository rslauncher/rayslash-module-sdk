# Module authoring

## Quick start

1. Copy `templates/wasm/` into a new public GitHub repository.
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

API v1 supports WASM modules. They implement `api/wit/rayslash-module.wit` and receive no WASI filesystem, process, or network API. Network/cache operations go through declared host capabilities. The `declarative` manifest value is reserved for a possible later API and is rejected by the v1 validator and registry.

The template README contains the exact build, binding-generation, local-validation, packaging, and release steps. Keep its WIT file synchronized with a released SDK API; do not edit the contract locally.

## Compatibility

`api_version` is a semantic-version requirement for the module API, not the launcher version. API 1.x keeps WIT and manifest behavior backward compatible. A breaking contract change creates API 2.

## Permissions

Declare only what the module needs. Network entries are exact HTTPS origins. Permission expansion during an update requires user confirmation. Command execution is high risk, typed, explicit-activation-only, and never evaluated by a shell.

`open-url` and `open-path` are typed activation requests executed by the launcher after selection. They are not query-time network or filesystem access. `open-path` accepts only a user-configured path supplied through validated module settings; executable modules cannot inspect that path themselves.

Timed behavior uses `schedule-notification` or `schedule-command`. The launcher owns the timer and executes only after explicit result activation. Modules must never implement delays by blocking a query or invoking a shell.

`host.unix-time` returns the current Unix timestamp in whole seconds. It is the only clock exposed to modules; use it for time calculations without depending on WASI or blocking the query.

## Stable IDs and results

Module IDs never change after publication. Provider IDs are stable within a module. Result IDs must identify the same logical item across queries so local learned ranking remains useful.

## Settings

The launcher owns settings storage and passes the module's validated settings object to each query as UTF-8 JSON in `query-context.settings-json`. Modules must treat missing fields as defaults and reject invalid values without panicking. Settings never grant permissions; permissions remain a separate install/update decision.
