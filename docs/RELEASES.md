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

- A commit with `feat:` or `feat(scope):` produces a **major update**:
  `1.1.1 -> 2.0.0`, resetting the minor and update counter.
- A commit with `fix:` or `fix(scope):` produces a **minor update**:
  `1.0.0 -> 1.1.1`, then `1.1.1 -> 1.2.2`.
- Maintenance changes to mod files without `feat:` or `fix:` increase
  only the update counter: `2.0.0 -> 2.0.1`.
- If several commits arrive in a single release, a feature takes
  precedence over a fix, and a fix takes precedence over maintenance.
- The third component counts update releases within the current major line.

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

Conventional Commit prefixes are essential for **automatic semantic
classification**. A change that adds a feature but lacks `feat:` will be
classified as maintenance; GitHub Actions cannot reliably infer intent
from a diff. Review commit titles before merging.

The release build also creates the pinned shared WASM standard, checks the
mod's Rust imports, and runs the compiled .smod through Sunrise's real Wasmtime
loader with a simulated host. Guest initialization, registration and hook
dispatch must pass **before** publishing. See [SANDBOX.md](SANDBOX.md).

This does not yet run actual world/gameplay scenarios or test against the
standard library packaged in a specific installed game. Those still require
separate validation.
