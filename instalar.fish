#!/usr/bin/env fish
# Build with the official pinned Rust; never replace the installed mod when ABI checks fail.
set -l base (status dirname)
cd "$base"; or exit 1

set -l rustroot "$HOME/.local/share/horizon-qol-rust"
set -l rustcargo "$rustroot/cargo"
set -l rustuphome "$rustroot/rustup"
if not test -x "$rustcargo/bin/rustup"
    echo 'Isolated official Rust toolchain not found.' >&2
    echo 'Run this first: fish preparar-rust.fish' >&2
    exit 1
end

set -l pacote target/sunrise-mod/horizon_qol_suite.smod
set -l std ""
if set -q SUNRISE_WASM_STD; and test -f "$SUNRISE_WASM_STD"
    set std "$SUNRISE_WASM_STD"
else
    set -l versoes "$HOME/.local/share/sunrise/launcher/versions"
    set -l found (find "$versoes" -maxdepth 2 -type f -path '*/nightly-*-2d81263/sunrise-wasm-std.wasm' -print 2>/dev/null)
    if test (count $found) -eq 1
        set std $found[1]
    end
end
if test -z "$std"
    echo 'Could not identify a unique Sunrise 2d81263 WASM standard library.' >&2
    echo 'Set SUNRISE_WASM_STD to the library used by your running Sunrise version.' >&2
    exit 1
end

begin
    set -lx CARGO_HOME "$rustcargo"
    set -lx RUSTUP_HOME "$rustuphome"
    set -lx PATH "$rustcargo/bin" "$HOME/.cargo/bin" $PATH
    if not type -q cargo-sunrise-mod
        echo 'cargo-sunrise-mod was not found in PATH.' >&2
        exit 1
    end

    # Verify the exact source revision and Rust compiler before generating a new package.
    rustup run 1.99.1 rustc -vV | grep -q '^release: 1.99.1$'; or exit 1
    echo 'Building with official Rust 1.99.1...'
    cargo sunrise-mod build --wasm; or exit 1
end

if not test -s "$pacote"
    echo "Missing build output: $pacote" >&2
    exit 1
end
if not unzip -tq "$pacote" >/dev/null
    echo 'Invalid ZIP package.' >&2
    exit 1
end
unzip -Z1 "$pacote" | grep -Fxq 'mod.wasm'
or begin
    echo 'The package does not contain mod.wasm.' >&2
    exit 1
end

set -l missing (python3 tools/wasm_link_check.py "$pacote" --std "$std" --json \
    | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["rust_symbols_not_exported_by_std"]))')
or exit 1

echo "Rust imports not resolved by the standard library: $missing"
if test "$missing" != 0
    echo 'Installation CANCELLED: the loader would reject this WASM module.' >&2
    echo "Verified standard library: $std" >&2
    echo 'Do not replace the standard library shipped with Sunrise.' >&2
    exit 1
end

set -l jogo "$HOME/.local/share/sunrise/game"
if set -q SUNRISE_HOME; and test -n "$SUNRISE_HOME"
    set jogo "$SUNRISE_HOME"
end
# Preserve previously registered values before the new private settings store takes over.
python3 tools/migrate_settings.py --game "$jogo"; or exit 1
set -l pasta "$jogo/sunrise-mods"
set -l config "$jogo/sunrise-mods.txt"
set -l destino "$pasta/horizon_qol_suite.smod"
mkdir -p "$pasta"; or exit 1
if test -f "$destino"
    cp "$destino" "$destino.bak"; or exit 1
end
cp "$pacote" "$destino"; or exit 1
if test -f "$config"
    cp "$config" "$config.bak"; or exit 1
    sed -i '/^mod\.horizon_qol_suite:/d' "$config"; or exit 1
end
printf '%s\n' 'mod.horizon_qol_suite:true' >> "$config"; or exit 1

echo "Installed package: $destino"
echo 'Restart Sunrise. Open Options > Mods > Sunrise QOL > Config > Open Settings, or press F8 in a world.'
