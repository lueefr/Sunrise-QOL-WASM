# Fast headless Sunrise validation

The workflows share `.github/actions/validate/action.yml` to check the exact
built `horizon_qol_suite.smod` using the SDK from the official Sunrise nightly.

## Validation

1. Python source checks and Rust unit tests.
2. Compile the mod with the signed, revision-matched Sunrise SDK and tool.
3. Check the `.smod` ZIP, bundled assets and guest imports against the
   matching WASM standard.
4. Load the actual `.smod` in Sunrise's `sunrise-mod-wasm` Wasmtime runtime
   and initialize the guest.

The smoke test does **not** dispatch gameplay events, create a game world,
open the graphical client or certify 1:1 behavior. Build/validation failure
blocks publishing, but manual gameplay checks remain necessary.

## Nightly cache rotation

Cache entries use the exact upstream commit SHA for the SDK, signed builder,
shared WASM standard and loader. Unchanged SDK versions reuse caches.
After a new SDK passes validation and is published, old revision-specific
caches are removed via GitHub's Actions Cache API. Rust dependencies are
also cached.

`sandbox.yml` handles pull requests and manual dispatch; `release.yml` handles
main releases. This avoids duplicate builds on main.

## Run locally

With Rust 1.99.1 and the revision-matched Sunrise builder installed:

```sh
python3 -m unittest discover -s tests -p 'test*.py'
cargo test --lib
cargo sunrise-mod build --wasm
cargo sunrise-mod wasm-std target/sunrise-wasm-std.wasm
python3 tools/sandbox/check.py target/sunrise-mod/horizon_qol_suite.smod target/sunrise-wasm-std.wasm
bash tools/sandbox/run.sh
```

The loader compiles against upstream source in `.sunrise/runtime-source` and
caches its dependencies in `target/sunrise-sandbox`. These checks verify
loading and binary compatibility, not actual Minecraft interactions.
