# Releases and version numbers

Releases run automatically on a push to `main` that changes the mod or its build inputs:
`src/**`, `assets/**`, `Cargo.toml`, `sunrise-mod.toml`, `instalar.fish`,
`preparar-rust.fish`, `diagnosticar.fish`, and the two runtime/build helpers
`tools/wasm_link_check.py` and `tools/migrate_settings.py`.

Changes only to `README.md`, `CHANGELOG.md`, `docs/**`, `tests/**`, the
release automation and other unlisted files **do not run a release**.
The release planner also checks the exact mod paths since the last tag,
so documentation-only changes do not bump the version.

The initial code release is **1.0.0**. Subsequent versions follow the
requested project-specific scheme (not standard SemVer):

- Each successful release increments exactly one decimal step, regardless of commit type:
  `1.1.4 -> 1.1.5 -> ... -> 1.1.9 -> 1.2.0 -> ... -> 1.9.9 -> 2.0.0`.
- An SDK update and changes to source files are combined into one release per cycle.
- A tag whose release assets are missing is resumed without incrementing again.

Examples of recommended commit subjects:

```text
feat: implement a new QoL module
fix: correct anvil maximum cost
refactor: reorganize module hooks
docs: clarify installation instructions
```

Changes to `main` are checked by Python tests, compiled as `.smod` using
`cargo sunrise-mod build --wasm`, and the package must contain `mod.wasm`.
Only successful jobs commit the version, create the `vX.Y.Z` tag and publish
an asset named `Sunrise-QOL-X.Y.Z.smod` with a changelog and SHA-256 checksum.

Conventional Commit prefixes help organize release notes but do not control
version increments. The version planner publishes one sequential update per cycle.

The release build also creates the pinned shared WASM standard, checks the
mod's Rust imports, and runs the compiled .smod through Sunrise's real Wasmtime
loader with a simulated host. Guest initialization, registration and hook
dispatch must pass **before** publishing. See [SANDBOX.md](SANDBOX.md).

This does not yet run actual world/gameplay scenarios or test against the
standard library packaged in a specific installed game. Those still require
separate validation.
