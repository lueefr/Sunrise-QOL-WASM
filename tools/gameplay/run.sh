#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
source="$root/.sunrise/runtime-source"
test -d "$source/.git" || { echo "Run tools/sandbox/run.sh first" >&2; exit 1; }
revision="$(python3 - <<'PY'
import tomllib
from pathlib import Path
print(tomllib.loads(Path("Cargo.toml").read_text())["package"]["metadata"]["sunrise"]["sdk"])
PY
)"
test "$(git -C "$source" rev-parse HEAD)" = "$revision" || { echo "Game revision differs from SDK pin" >&2; exit 1; }
smod="$root/target/sunrise-mod/horizon_qol_suite.smod"
standard="$root/target/sunrise-wasm-std.wasm"
test -s "$smod" && test -s "$standard" || { echo "Missing actual .smod or matching WASM std" >&2; exit 1; }
cp "$root/tools/gameplay/sunrise_qol_gameplay.rs.txt" "$source/crates/sunrise-client/src/game/modding/sunrise_qol_gameplay.rs"
game_rs="$source/crates/sunrise-client/src/game.rs"
if ! grep -Fq 'mod sunrise_qol_gameplay;' "$game_rs"; then
  printf '\n%s\n%s\n%s\n' '#[cfg(test)]' '#[path = "game/modding/sunrise_qol_gameplay.rs"]' 'mod sunrise_qol_gameplay;' >> "$game_rs"
fi
export SUNRISE_QOL_SMOD="$(realpath "$smod")"
export SUNRISE_WASM_STD="$(realpath "$standard")"
export CARGO_TARGET_DIR="$root/target/sunrise-sandbox"
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_PROFILE_DEV_DEBUG=0
export RUST_BACKTRACE=1
mkdir -p "$root/target"
log="$root/target/gameplay-run.log"
report="$root/target/gameplay-evidence.json"
rm -f "$log" "$report"
echo "Compiling the real sunrise-client Game integration test"
if cargo test --manifest-path "$source/Cargo.toml" --locked --package sunrise-client \
  --lib --no-default-features --features testing --no-run 2>&1 | tee "$log"; then
  echo "PASS: real-game Rust harness compiled" | tee -a "$log"
else
  echo "QOL_GAMEPLAY_BLOCKED|engine-compile-failure" | tee -a "$log"
  python3 tools/gameplay/report.py --log "$log" --output "$report" \
    --exit-code 1 --source "$revision" --package "$smod"
  exit 1
fi

# Sunrise intentionally does not distribute Mojang's prepared game files.
# The public GitHub runner must NEVER silently fall back to mocked assets.
resources="$source/.local/sunrise-client/26.3"
if [ -n "${SUNRISE_TEST_RESOURCES:-}" ]; then
  resources="$SUNRISE_TEST_RESOURCES"
fi
if [ ! -f "$resources/facts.json" ] ||
   [ ! -f "$resources/gameplay-facts.json" ] ||
   [ ! -f "$resources/block-behaviours.json" ]; then
  echo "QOL_GAMEPLAY_BLOCKED|missing-resources" | tee -a "$log"
  echo "::error::Real-game parity cannot run: prepared Minecraft 26.3 resources missing at $resources" >&2
  echo "Provision a licensed Sunrise prepared-data folder on a private runner." >&2
  echo "Set repository variables SUNRISE_QOL_GAMEPLAY_RUNNER and SUNRISE_TEST_RESOURCES." >&2
  echo "A .smod/Wasmtime smoke pass is not a gameplay parity pass." >&2
  python3 tools/gameplay/report.py --log "$log" --output "$report" \
    --exit-code 2 --source "$revision" --package "$smod" 
  exit 1
fi
export SUNRISE_TEST_RESOURCES="$(realpath "$resources")"
echo "Testing Game revision $revision using prepared game data"
if cargo test --manifest-path "$source/Cargo.toml" --locked --package sunrise-client --lib \
  --no-default-features --features testing sunrise_qol_real_game_soil_parity \
  -- --nocapture --test-threads=1 2>&1 | tee -a "$log"; then
  status=0
else
  status=1
fi
python3 tools/gameplay/report.py --log "$log" --output "$report" --exit-code "$status" \
  --source "$revision" --package "$smod"
