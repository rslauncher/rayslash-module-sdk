# RaySlash module API v1

The canonical contract is [`api/wit/rayslash-module.wit`](../api/wit/rayslash-module.wit). A module is a WebAssembly component exporting `provider.query` and importing only `host`; the host intentionally supplies no WASI interfaces.

## Query contract

`query-context` contains the current UTF-8 query, a result limit from 1 through 100, an optional locale, and launcher-managed module settings serialized as JSON. Community modules must currently work with `{}` because API v1 has no generic community settings form. Queries must return promptly, must not implement their own delay, and should return an empty non-exclusive response when they do not match.

Each result has a stable module-local ID, title, subtitle, optional score, icon, and one typed action. `exclusive = true` suppresses core Apps/Folders results for that query. Text and result counts are bounded again by the host and launcher.

Icons may be short text, `none`, or a safe relative package path. Package paths cannot be absolute or contain parent traversal.

## Typed actions

- `copy-text`, `open-url`, `open-path`, and `show-message` run only after explicit result activation.
- `notify` requires notification permission.
- `run-approved-command` is an argument vector, never a shell string, and requires command permission.
- `schedule-notification` and `schedule-command` contain a delay in whole seconds. The launcher owns scheduling.
- `none` is a preview-only result.

An `open-path` target must exactly occur in the validated settings JSON supplied for that query. Command and notification requests without their declared/granted permissions are rejected by the launcher.

## Host capabilities

- `request` permits only `GET` or `POST` to exact HTTPS origins declared in `module.toml`. Authorization, cookie, proxy, host, and connection headers are rejected. Requests and responses have time and size limits.
- `cache-get` and `cache-put` use module-private keys containing only ASCII letters, digits, `.`, `_`, or `-`. Values are bounded and writes are atomic.
- `unix-time` returns whole seconds since the Unix epoch. No ambient clock, environment, filesystem, socket, or process API exists.

## Errors and limits

Use `invalid-query` for a matched but invalid query, `unavailable` for a temporary dependency failure, and `internal` only for unexpected module failure. Do not include secrets, credentials, or private paths in errors.

The production host applies fuel, a default 32 MiB linear-memory limit, a 20,000,000 fuel budget, bounded HTTP/cache values, and launcher process deadlines. Modules must still cap their own parsing and allocation. A trap or malformed response terminates only the disposable module host.

## Compatibility

`api_version` is a semantic-version requirement against the module API, not the app version. Publish a new immutable module version for every package change. Never replace a release asset already submitted to the registry.
