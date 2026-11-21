# Local validation of the PR 8 review

The reviewed package targets Sunrise SDK `94256510f74e6843754e0a93fd9e4da7e45b1056` (nightly 2026-11-20). The previous SDK build was refused by this runtime because API signatures had changed.

Validated locally:

- 66 native Rust helper tests and 107 Python source/configuration tests pass.
- `cargo-sunrise-mod check` accepts the compiled package.
- `tools/wasm_link_check.py` finds no missing Rust imports in the matching WASM standard library.
- The package loads in the real client with one WASM mod enabled.
- Nine Automatic Doors scenarios pass in a disposable integrated-server world. The JSON records both door halves and the exact package SHA-256.

These observations do **not** establish full parity: they do not compare against the original Java mod running in the same fixtures, test multiplayer prediction, or cover all modules and options. The source audit and parity matrix retain their partial statuses. The existing CI Game suite also needs prepared game resources; local observations do not resolve its runner setup.

## Repeat the door observations

Start the matching client with `--developer-single-player --offscreen`, `SUNRISE_STILL=1`, the matching `SUNRISE_WASM_STD`, the reviewed package enabled, a fresh game directory and a **disposable** world. Wait until the world is loaded. Close any UI before running:

```sh
python tools/gameplay/control_doors.py \
  --ctl /path/to/sunrise-ctl \
  --socket /tmp/qol-review/ctl.sock \
  --game-dir /tmp/qol-review/game \
  --package target/sunrise-mod/horizon_qol_suite.smod \
  --output /tmp/qol-review/automatic-doors.json
```

The probe changes blocks near `(0, 91, 0)`, player position/game mode and Automatic Doors settings. Use no real save or user configuration. It checks disabled behavior, both-half opening, integer-second delay, closure on departure, reopening, keeping doors open while nearby, departure after expiry, and the iron-door switch in both positions.

## Listing assets

`../registry/menu.png` is one unedited screenshot of the actual menu in the disposable client, with modules off. `../registry/horizon_qol_suite.png` is an original 512×512 PNG icon (under 256 KiB), rendered from the adjacent SVG. These are prepared assets, not evidence of a published sunrise-mods listing.

Publication remains pending full parity, release validation and a license choice for this project's own code. The third-party MIT notice is not a license for the entire project.
