# Local validation of PR 8 and the next mod batch

The reviewed package targets Sunrise SDK `94256510f74e6843754e0a93fd9e4da7e45b1056` (nightly 2026-11-20). The previous SDK build was refused by this runtime because API signatures had changed.

Validated locally:

- 71 native Rust helper tests and 107 Python source/configuration tests pass for the next-mod batch (PR #8 had 66 Rust tests).
- `cargo-sunrise-mod check` accepts the compiled package.
- `tools/wasm_link_check.py` finds no missing Rust imports in the matching WASM standard library.
- The package loads in the real client with one WASM mod enabled.
- Nine Automatic Doors scenarios passed for the PR #8 package; its JSON records both door halves and that package SHA-256. 21 Wool Tweaks interaction scenarios cover disabled behavior, survival/adventure/creative, same-color no-op, carpets, unrelated blocks, moss carpet, offhand use and bed pairs in all four directions, plus serialized occupation state. 11 recipe scenarios verify admission of all 49 definitions, disable/reload, native preview and actual ingredient consumption for wool, beds, carpet rings and wool-to-string conversion. Each JSON identifies its own tested package hash.

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

## Repeat the Wool Tweaks observations

In the same kind of disposable developer world, run `tools/gameplay/control_wool.py` with the same five arguments as the door probe. It changes player inventory/game mode, difficulty, blocks near `(2, 91, 0)` and the private Wool Tweaks switch. The module is switched OFF on exit. Evidence is in `wool-9425651.json`. Bed occupation uses synthetic block state, not an actually sleeping player. Recipes have a separate probe: `control_wool_recipes.py`, with the same five arguments. This also uses `/reload`, changes recipe-book knowledge and tests crafting slots. Its evidence is `wool-recipes-9425651.json`. The original recipe structures and blob hashes are compared separately in `wool-recipe-contracts.json`. Recipe admission requires `/reload` after toggle/world entry and is not certified 1:1.

## Cultivation and optional recipes

`cultivation-9425651.json` records 14 real-game scenarios at SDK 9425651: disabled/enabled recipe admission, bucket smelting with real fuel/time, kelp growth/consumption, leggings and unrelated damage, Feather Falling and Slow Falling farmland landings, and unprotected farmland converting to dirt. Reproduce with `tools/gameplay/control_cultivation.py` in a disposable world, passing `--ctl`, `--socket`, `--game-dir`, `--package` and `--output`. The damage-source command deliberately tests the broader source-filter adaptation; actual moving-through-bush slowing and multiplayer are not certified. Recipe admission is not a test of every recipe crafting layout.

`convenience-9425651.json` adds ten scenarios on the next package: hand refill and disabled control, bright mushroom placement and disabled rejection, early/allowed sleep clock, boat/cart carrying and release with unchanged UUID. Run `tools/gameplay/control_convenience.py` with the same arguments. Vehicles are tested through registered server commands because the client did not expose their hit targets. Chest-vehicle container commands were refused by this nightly, so cargo functionality is not certified. The cultivation evidence belongs to its explicitly hashed earlier package; its source was unchanged in the next package.

`paths-9425651.json` records five real Quick Paths scenarios on the final package: disabled vanilla-only conversion, selection without terrain changes, four-block route, four durability consumed, and endpoint undo. Reproduce with `tools/gameplay/control_paths.py` using the same disposable-world arguments. Terrain-following, long routes, blocked columns and multiplayer are not certified by these five cases.
