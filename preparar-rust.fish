#!/usr/bin/env fish
# Install the pinned official Rust toolchain in a private directory, never via pacman.
set -l root "$HOME/.local/share/horizon-qol-rust"
set -l cargohome "$root/cargo"
set -l rusthome "$root/rustup"
mkdir -p "$root"; or exit 1

if not test -x "$cargohome/bin/rustup"
    if not type -q curl
        echo 'curl was not found.' >&2
        exit 1
    end
    curl --fail --location --proto '=https' --tlsv1.2 \
        --output "$root/rustup-init" \
        'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init'
    or exit 1
    chmod 700 "$root/rustup-init"; or exit 1
    begin
        set -lx CARGO_HOME "$cargohome"
        set -lx RUSTUP_HOME "$rusthome"
        "$root/rustup-init" -y --no-modify-path --profile minimal --default-toolchain none
        or exit 1
    end
end

begin
    set -lx CARGO_HOME "$cargohome"
    set -lx RUSTUP_HOME "$rusthome"
    set -lx PATH "$cargohome/bin" $PATH
    rustup toolchain install 1.99.1 --profile minimal --component rust-src --target wasm32-unknown-unknown
    or exit 1
    echo 'Official Rust toolchain installed in an isolated directory.'
    rustup run 1.99.1 rustc -vV
end
