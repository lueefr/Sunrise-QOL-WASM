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

The runner performs a Rust `cargo check --tests` of the real Game integration
harness even when private prepared resources are unavailable, so type errors
are still detected without spending time on full native linking. Full Game
compilation and gameplay execution happen only when game data exist.
If resources are missing the job fails with a machine-readable
`blocked_reason=missing-prepared-game-resources` instead of skipping or
passing a fake test. PRs retain an explicitly **unverified** .smod artifact
for manual testing without treating the artifact as a release.

## Installed private runner

The local runner uses rootless Podman with a fresh ephemeral GitHub registration per job. It mounts only the prepared `26.3` resource directory read-only at `/opt/sunrise-resources/26.3`; the personal home, SSH credentials and container daemon sockets are not mounted. The host wrapper obtains a short-lived registration token using the existing GitHub CLI login. It passes that token in a mode-0600 temporary file and removes the file when the container exits. The runner removes the token from its environment before accepting jobs.

Repository variables: `SUNRISE_QOL_GAMEPLAY_RUNNER=sunrise-qol-private`, `SUNRISE_TEST_RESOURCES=/opt/sunrise-resources/26.3`. The sandbox job runs only for repository-owned branches; external contributions must be reviewed and brought to a maintainer branch. Repository policy also requires approval for every external contributor's workflow run. Do not remove those controls or approve unreviewed workflows that target the private runner.

Reproducible container sources are in `tools/runner/`. Download the official Linux x64 runner archive to a private build directory as `actions-runner.tar.gz` and verify its SHA256 using GitHub's runner-downloads API before building this image. Copy the Containerfile and entrypoint into that directory; build with `podman build -t localhost/sunrise-qol-runner:latest DIRECTORY`. Never add the runner archive, registration credentials or game resources to Git.

Install `tools/runner/run.sh` under the local runner directory. It reads `~/.config/sunrise-qol-runner/config.sh` containing `SUNRISE_RESOURCES`, `SUNRISE_RUNNER_REPOSITORY` and optional `SUNRISE_RUNNER_NAME`. Run through the user systemd service `sunrise-qol-runner.service`; `Restart=always` creates a clean container and ephemeral registration after each job. Jobs are limited to two CPUs, 6 GiB memory and 8 GiB memory plus swap. The computer must remain online and its user systemd session must remain active.

The proprietary resource mount is excluded from repository caches and uploaded artifacts. CI uploads only the mod, test logs and structured evidence. Missing resources still fail the real-game gate.
