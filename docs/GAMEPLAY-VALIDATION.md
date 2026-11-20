# In-game source parity validation

The workflow now has two different tiers of runtime validation, for the **same**
built .smod and exact Sunrise nightly SDK revision:

1. Wasmtime guest loading and hook dispatch with simulated host data
   (tools/sandbox/run.sh). This is useful for binary and API compatibility,
   but is **not** an in-game test.
2. Actual Sunrise Game world tests (tools/gameplay/run.sh): inject a test
   module into the SDK-pinned sunrise-client crate, install the production
   WASM guest, perform player actions through the game engine and assert
   actual resulting world state. Failure prevents release tagging and publish.

The first source-derived contract is [Keep My Soil Tilled, upstream 26.3.0](https://github.com/Serilum/Keep-My-Soil-Tilled/blob/26.3.0/Common/src/main/java/com/serilum/keepmysoiltilled/events/StemBlockHarvestEvent.java). It checks the disabled module, harvesting a melon, harvesting a pumpkin, no neighboring farmland and breaking an unrelated block. Tests verify the changed world blocks **after a real player break**. A callback-only mock can never satisfy this test.

The real-game runner writes gameplay-run.log and gameplay-evidence.json. The
attestation rejects nonzero process status, missing, repeated or unexpected
case IDs, FAIL results, or an absent Rust test success line. Evidence artifacts
are uploaded even on failures.

**Limitations:** A real Game library fixture is a headless engine test, not a
graphical client session. Visual parity requires capture tests. Successful
engine cases are necessary evidence but insufficient to prove the **entire**
module is 1:1, especially mod interactions, RNG order, multiplayer client
prediction and untested configuration combinations. The report explicitly
tracks the remaining modules without gameplay coverage and never claims a
fully certified 1:1 module. Expand cases.json and the actual Game fixture for
each additional contract derived from the original implementation.
