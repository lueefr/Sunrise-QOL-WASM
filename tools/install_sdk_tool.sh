#!/usr/bin/env bash
# Install the tool built for the exact published Sunrise SDK, not a tool from a stale Git commit.
# Verification follows sunrise-org/sunrise/docs/modding/src/start/install-tool.md.
set -euo pipefail

revision="$1"
if [[ ! "$revision" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Invalid Sunrise SDK revision: $revision" >&2
  exit 1
fi

release="sdk-$revision"
base="https://github.com/sunrise-org/sunrise/releases/download/$release"
tool="tool-x86_64-unknown-linux-gnu.tar.xz"
# Sunrise's release signing key: crates/sunrise-release-sign/release-keys.txt
key="r53xb3cuafGxZSicLz06KC+xXp9NJdPH8snNmXo5pAc="
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

for name in "$tool" SHA256SUMS SHA256SUMS.sig; do
  curl --fail --silent --show-error --location --retry 4 --retry-delay 2 \
    "$base/$name" -o "$name"
done

# The official signature covers the release identity AND its checksum manifest.
printf 'horizon release %s\n' "$release" | cat - SHA256SUMS > message
{
  printf '\060\052\060\005\006\003\053\145\160\003\041\000'
  printf '%s' "$key" | openssl base64 -d -A
} > key.der

verified=false
while IFS= read -r line || [[ -n "$line" ]]; do
  [[ -z "$line" ]] && continue
  printf '%s' "$line" | openssl base64 -d -A > signature
  if openssl pkeyutl -verify -rawin -pubin -keyform DER \
      -inkey key.der -in message -sigfile signature >/dev/null 2>&1; then
    verified=true
    break
  fi
done < SHA256SUMS.sig
if [[ "$verified" != true ]]; then
  echo "Sunrise release $release has no valid signature" >&2
  exit 1
fi

# Verify exactly the archive we're about to install, not an unrelated asset.
awk -v name="$tool" '$2 == name {print}' SHA256SUMS > expected
if [[ "$(wc -l < expected)" -ne 1 ]]; then
  echo "Missing or duplicate checksum for $tool" >&2
  exit 1
fi
sha256sum --check expected

# SDK tool archives may prefix the executable path with a directory.
# Extract the signed archive and locate the one expected binary by name.
tar -xJf "$tool"
binary="$(find "$tmp" -type f -name cargo-sunrise-mod -print -quit)"
if [[ -z "$binary" ]]; then
  echo "Signed SDK tool archive has no cargo-sunrise-mod executable" >&2
  exit 1
fi
install -Dm755 "$binary" "$HOME/.cargo/bin/cargo-sunrise-mod"
test -x "$HOME/.cargo/bin/cargo-sunrise-mod"
echo "Installed signed Sunrise build tool for $revision"
