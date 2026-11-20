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
echo "Testing actual Game at Sunrise revision $revision"
if cargo test --manifest-path "$source/Cargo.toml" --locked --package sunrise-client --lib \
  --no-default-features --features testing sunrise_qol_real_game_soil_parity \
  -- --nocapture --test-threads=1 2>&1 | tee "$log"; then
  status=0
else
  status=1
fi
python3 tools/gameplay/report.py --log "$log" --output "$report" --exit-code "$status" \
  --source "$revision" --package "$smod"
