#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

revision="$(python3 - <<'PY'
from pathlib import Path
import tomllib
with Path("Cargo.toml").open("rb") as f:
    print(tomllib.load(f)["package"]["metadata"]["sunrise"]["sdk"])
PY
)"

if [[ ! "$revision" =~ ^[[:xdigit:]]{40}$ ]]; then
  echo "Invalid pinned Sunrise revision: $revision" >&2
  exit 1
fi

package="${1:-$root/target/sunrise-mod/horizon_qol_suite.smod}"
standard="${2:-$root/target/sunrise-wasm-std.wasm}"
test -s "$package" || { echo "Missing .smod package: $package" >&2; exit 1; }
test -s "$standard" || { echo "Missing pinned WASM std: $standard" >&2; exit 1; }

# This binary is keyed by the exact SDK SHA, Rust version and smoke-test source
# in the CI cache. It loads the current .smod instead of rebuilding Sunrise.
export SUNRISE_QOL_SMOD="$(realpath "$package")"
export SUNRISE_WASM_STD="$(realpath "$standard")"
loader="$root/.sunrise/loader-cache/sunrise_qol_smoke"
if [[ -x "$loader" ]]; then
  echo "Reusing precompiled Sunrise Wasmtime loader for $revision"
  "$loader" --exact sunrise_qol_loads_and_initializes --nocapture
  exit 0
fi

source="$root/.sunrise/runtime-source"
mkdir -p "$source"
if [[ ! -d "$source/.git" ]]; then
  git -C "$source" init -q
  git -C "$source" remote add origin https://github.com/sunrise-org/sunrise.git
fi
# Reuse the same source only if its exact object matches the SDK revision.
if ! git -C "$source" cat-file -e "$revision^{commit}" 2>/dev/null; then
  git -C "$source" -c http.version=HTTP/1.1 fetch --quiet --depth 1 --filter=blob:none origin "$revision"
fi
git -C "$source" checkout -q --force --detach "$revision"
test "$(git -C "$source" rev-parse HEAD)" = "$revision"

cp "$root/tools/sandbox/sunrise_qol_smoke.rs.txt" \
   "$source/crates/sunrise-mod-wasm/tests/sunrise_qol_smoke.rs"

export CARGO_TARGET_DIR="$root/target/sunrise-sandbox"
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-3}"
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_PROFILE_DEV_DEBUG=0
export RUST_BACKTRACE=1

echo "Using Sunrise runtime commit $revision"
echo "Running Sunrise Wasmtime loading and registration smoke test"
cargo test --manifest-path "$source/Cargo.toml" --locked \
  --package sunrise-mod-wasm --test sunrise_qol_smoke -- \
  --exact sunrise_qol_loads_and_initializes --nocapture

# Store only the verified smoke executable, not the multi-gigabyte Rust target.
binary="$(find "$CARGO_TARGET_DIR/debug/deps" -maxdepth 1 -type f \
  -name 'sunrise_qol_smoke-*' -perm /111 -print -quit)"
test -n "$binary" || { echo "Sunrise smoke test executable not found" >&2; exit 1; }
install -Dm755 "$binary" "$loader"
echo "Prepared Sunrise loader binary for the SDK cache"
