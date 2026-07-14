# Module authoring

## Quick start

Start from the immutable SDK release and copy the template into an empty working directory:

```sh
git clone --depth 1 --branch v1.0.1 https://github.com/rslauncher/rayslash-module-sdk.git
mkdir my-rayslash-module
cp -a rayslash-module-sdk/templates/wasm/. my-rayslash-module/
cd my-rayslash-module
```

Then:

1. Create a new public GitHub repository for this directory.
2. Choose a globally unique reverse-DNS module ID. Community modules cannot use `rayslash.*` or author `rayslash`.
3. Complete `module.toml`, `README.md`, `LICENSE`, and the local icon.
4. Follow the checked-in template README's build/test commands, then copy the release component to `module.wasm` beside `module.toml`.
5. Validate locally:

   ```sh
   cargo run --manifest-path ../rayslash-module-sdk/Cargo.toml \
     -p rayslash-module-tool --locked -- validate .
   ```

6. Package locally:

   ```sh
   cargo run --manifest-path ../rayslash-module-sdk/Cargo.toml \
     -p rayslash-module-tool --locked -- package .
   ```

7. Publish the `.tar.zst` and `.sha256` files as immutable GitHub Release assets.
8. Submit the repository through a pull request to `rslauncher/rayslash-registry`.

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

The launcher owns settings storage and passes any launcher-managed settings object to each query as UTF-8 JSON in `query-context.settings-json`. Modules must treat `{}`, missing fields, and invalid values as safe defaults and must not panic. API v1 does not define a community-module settings schema or generic settings form yet, so a community module must remain useful with `{}`. Settings never grant permissions; permissions remain a separate install/update decision.
