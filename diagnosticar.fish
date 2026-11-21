#!/usr/bin/env fish
# Inspect the built WASM package and the standard-library member of the running game.
set -l base (status dirname)
set -l pacote "$base/target/sunrise-mod/sunrise_qol.smod"
if not test -f "$pacote"
    echo "Build the mod first with: fish instalar.fish" >&2
    exit 1
end
set -l versoes "$HOME/.local/share/sunrise/launcher/versions"
set -l stds (find "$versoes" -type f -name 'sunrise-wasm-std.wasm' 2>/dev/null | sort -r)
if test (count $stds) -eq 0
    echo "Could not find sunrise-wasm-std.wasm in installed versions. Showing imports only."
    python3 "$base/tools/wasm_link_check.py" "$pacote"
else
    echo "Found standard library member: $stds[1]"
    python3 "$base/tools/wasm_link_check.py" "$pacote" --std "$stds[1]"
end
