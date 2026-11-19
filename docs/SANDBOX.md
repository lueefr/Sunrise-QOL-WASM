# Headless Sunrise sandbox

This pipeline tests the exact packaged `horizon_qol_suite.smod` against the
actual `sunrise-mod-wasm` loader at the **same Git revision as the pinned SDK**.
It does not replace a full in-game acceptance test.

## Validation levels

1. `python3 -m unittest discover -s tests` checks the source regressions
   and the binary import checker using test-built WebAssembly sections.
2. `cargo sunrise-mod build --wasm` compiles the QOL guest to `.smod`.
3. `cargo sunrise-mod wasm-std target/sunrise-wasm-std.wasm` creates a shared
   standard library from the pinned compiler SDK. The `check.py` stage reports
   imported Rust symbols not exported by that standard or the guest itself.
4. `bash tools/sandbox/run.sh` checks out the exact Sunrise source commit,
   compiles its real `sunrise-mod-wasm` runtime with Wasmtime, loads the
   **built .smod**, executes guest initialization/registration and calls the
   mod's `server_stopped` hook through the real Wire transport.

Any failure in steps 2–4 prevents `release.yml` from tagging or publishing
a build. The report is attached as an Actions artifact.

The independent [sandbox workflow](../.github/workflows/sandbox.yml) runs
automatically when its own implementation changes and can also be dispatched
manually. It does not publish a release.

## Run locally

With Rust 1.99.1, the Sunrise mod builder and its pinned SDK prepared:

```sh
cargo sunrise-mod build --wasm
cargo sunrise-mod wasm-std target/sunrise-wasm-std.wasm
python3 tools/sandbox/check.py target/sunrise-mod/horizon_qol_suite.smod target/sunrise-wasm-std.wasm
bash tools/sandbox/run.sh
```

The native runtime is built separately (cached in `target/sunrise-sandbox`).
The upstream source checkout is in `.sunrise/runtime-source`, pinned to the
exact SDK hash from `Cargo.toml`; it is not vendored into this repository.

## What this *does not* prove

- The generated std member is built from the pinned SDK, **not necessarily
  byte-identical** to the std shipped by a specific launcher/nightly installation.
  Testing against the installed game still matters.
- The smoke test's host is a minimal simulated server host, **not a populated
  Minecraft world**. It proves that the real guest can link, initialize, register,
  and execute at least one real hook over Wasmtime. It does not certify that
  combat, world changes, inventories, GUI rendering or multiplayer gameplay works.
- Future gameplay coverage should use Sunrise's existing headless `Game` test
  fixtures to create players, blocks and entities, then assert world state after
  dispatching each mod's hooks. Such tests must explicitly enable the relevant
  module and inspect effects; merely finding a module toggle is not a passing test.

If the real loader test becomes too heavy for the GitHub-hosted runner, investigate
runtime dependency caching or a larger runner instead of silently skipping it.
