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
   compiles its real `sunrise-mod-wasm` runtime with Wasmtime, and loads the
   **built .smod**. It executes guest initialization/registration, dispatches
   gameplay hooks with an in-memory configuration/player/entity-event host,
   asserts the responses, and finally calls `server_stopped` through the
   real Wire transport.

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

## Behavioral regression scenarios (real guest, simulated host)

These tests invoke Sunrise's generated hook dispatchers on the packaged guest,
not Rust methods copied into a test library. Each scenario checks **observable
answers**, and the test fails when an assertion does not hold:

| Module | Stimulus | Expected result |
| --- | --- | --- |
| Compact Help | Five permitted commands; module OFF | Vanilla `/help` continues, no synthetic replies |
| Compact Help | Module ON, page size 2; `/help 1`, `/help 2` | Sorted two-command pages, correct clickable suggestions, previous/next navigation |
| Compact Help | `/help give` | Detailed help remains handled by vanilla |
| Extended Enchant Command | Module OFF; high `/enchant @s` | Vanilla handles it; QOL does not cancel |
| Extended Enchant Command | Module ON, max level 10; `/enchant @s ... 11` | Rejects command with an explicit limit message |
| Extended Enchant Command | Selector `@a` | Vanilla command path preserved |
| Friendly Griefing | Villager join; module OFF | Existing `mobGriefing=false` preserved |
| Friendly Griefing | Villager join; module ON | Only the tracked villager receives `can_grief=true`, not a zombie |
| Friendly Griefing | Villager leaves level | The previous entity ID loses its special permission |

**Coverage:** 3 of 38 implemented modules have direct behavioral sandbox
scenarios at this stage. Other modules are still compiled and smoke-loaded,
but their gameplay effects are **not yet automatically verified**. This figure
does not count source-pattern unit tests as gameplay coverage.

The test host is `crates/sunrise-mod-wasm/tests/support/bulk_files.rs`
from the pinned Sunrise source tree. It provides real in-memory config
storage and dispatches entity lifecycle/command events through Wasmtime;
it is **not** the full Sunrise `Game` or a populated level.

## What this *does not* prove

- The generated std member is built from the pinned SDK, **not necessarily
  byte-identical** to the std shipped by a specific launcher/nightly installation.
  Testing against the installed game still matters.
- The behavioral test host is in-memory, **not a populated Minecraft world**.
  It verifies the actual command responses and selected entity-rule decisions.
  It cannot yet certify combat, block mutation, inventory changes, GUI rendering,
  mob spawning, dimension transitions or multiplayer gameplay.
- Future world-state tests should use Sunrise's existing headless `Game` test
  fixtures to create players, blocks and entities, then assert changes after
  dispatching QOL's hooks. Such tests must explicitly enable the relevant
  module and inspect the actual world state; merely finding a toggle is not
  sufficient.

If the real loader test becomes too heavy for the GitHub-hosted runner, investigate
runtime dependency caching or a larger runner instead of silently skipping it.
