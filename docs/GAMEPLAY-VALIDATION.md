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

## Required game data and runner

Sunrise's original real-Game unit tests require a private, locally prepared
Minecraft 26.3 data tree. The Sunrise upstream daily CI uses self-hosted
runners with an owned Minecraft copy; it does not publish that tree. The
Game test deliberately refuses to run without facts.json,
gameplay-facts.json and block-behaviours.json and will **block the release**
instead of passing synthetic tests as game evidence.

To enable the real-game gate in GitHub Actions, provision an authorized
private runner with Sunrise-prepared game resources. Set the repository
variables SUNRISE_QOL_GAMEPLAY_RUNNER to its runner label and
SUNRISE_TEST_RESOURCES to the absolute path of its private prepared game
directory. The default hosted runner cannot prove gameplay parity until such
data are legitimately available. Do not commit or upload proprietary
Minecraft game data into the repository or public CI artifacts.

This is a fail-closed requirement. A green synthetic-host build alone is not
an in-game verification.

The runner compiles the real Game integration harness even when private
prepared resources are unavailable, so code errors are still detected.
If resources are missing the job fails with a machine-readable
`blocked_reason=missing-prepared-game-resources` instead of skipping or
passing a fake test. PRs retain an explicitly **unverified** .smod artifact
for manual testing without treating the artifact as a release.
