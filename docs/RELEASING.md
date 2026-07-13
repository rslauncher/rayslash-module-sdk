# Release and registry submission

1. Run formatting, tests, clippy, and `cargo component build --release --target wasm32-unknown-unknown` with the pinned toolchain. Run `cargo fmt` after binding generation, then finish with `cargo fmt --check` and `git diff --exit-code`; the tracked generated bindings are formatted for the pinned toolchain and a clean documented build must not rewrite them.
2. Copy the component to `module.wasm` beside `module.toml`, `README.md`, `LICENSE`, and the icon.
3. Run `rayslash-module validate` and `rayslash-module package`. The package command creates a deterministic `<id>-<version>.tar.zst` plus SHA-256 file.
4. Tag the exact source commit and upload both files to a public GitHub Release. Tags and assets are immutable; corrections use a higher semantic version.
5. Fork `rslauncher/rayslash-registry`, add one submission TOML, and open a pull request. Include the source commit, release URL, byte size, digest, API requirement, permissions, and review status requested by the registry template.

Pull-request validation has no production signing secret. It downloads the pinned asset, enforces archive and manifest rules, and checks identity, permissions, size, and digest. After maintainer review and merge, the protected production workflow signs and publishes the new catalog.

Permission expansion is visible to users and requires approval during update. Minimize permissions and explain every network origin or high-risk action in the README. Security reports belong in the affected repository's private security advisory, never a public issue containing exploit details.
