#!/usr/bin/env python3
"""Pin the SDK of Sunrise's published nightly, never an unpublished main commit."""
from __future__ import annotations

import argparse
import json
import os
import re
import tomllib
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
API = "https://api.github.com/repos/sunrise-org/sunrise"
SHA = re.compile(r"[0-9a-f]{40}\Z")
SDK_LINE = re.compile(r'(?m)^(sdk\s*=\s*")[0-9a-f]{40}(")[ \t]*


def read_api(endpoint: str) -> dict:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "sunrise-qol-sdk-sync",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = Request(API + endpoint, headers=headers)
    try:
        with urlopen(request, timeout=25) as response:
            return json.load(response)
    except (HTTPError, URLError, ValueError) as error:
        raise ValueError(f"Could not verify Sunrise {endpoint}: {error}") from error


def latest(fetch=read_api) -> str:
    """Resolve the signed SDK for the exact commit of the official nightly."""
    nightly = fetch("/releases/tags/nightly")
    ref = fetch("/git/ref/tags/nightly")
    revision = nightly.get("target_commitish", "")
    if nightly.get("tag_name") != "nightly" or nightly.get("draft"):
        raise ValueError("No published Sunrise nightly")
    if not isinstance(revision, str) or not SHA.fullmatch(revision):
        raise ValueError("Sunrise nightly does not identify a full commit SHA")
    obj = ref.get("object", {})
    if obj.get("type") != "commit" or obj.get("sha") != revision:
        raise ValueError("Sunrise nightly release and tag point to different commits")

    sdk = fetch(f"/releases/tags/sdk-{revision}")
    if (sdk.get("tag_name") != f"sdk-{revision}" or sdk.get("draft")
            or sdk.get("target_commitish") != revision):
        raise ValueError(f"SDK release for nightly {revision} is not published")
    assets = {asset.get("name") for asset in sdk.get("assets", [])}
    required = {"sdk.json", "SHA256SUMS", "SHA256SUMS.sig",
                "api-x86_64-unknown-linux-gnu.tar.xz"}
    if not required.issubset(assets):
        raise ValueError(f"SDK {revision} is incomplete: missing {sorted(required - assets)}")
    return revision


def pinned(cargo: Path) -> str:
    data = tomllib.loads(cargo.read_text(encoding="utf-8"))
    revision = data["package"]["metadata"]["sunrise"]["sdk"]
    if not isinstance(revision, str) or not SHA.fullmatch(revision):
        raise ValueError("Cargo.toml has an invalid Sunrise SDK pin")
    return revision


def repin(cargo: Path, revision: str) -> tuple[str, bool]:
    if not SHA.fullmatch(revision):
        raise ValueError("Cannot pin an invalid Sunrise revision")
    before = pinned(cargo)
    if before == revision:
        return before, False
    original = cargo.read_text(encoding="utf-8")
    updated, count = SDK_LINE.subn(lambda match: match.group(1) + revision + match.group(2),
                                   original)
    if count != 1:
        raise ValueError("Expected one SDK pin in Cargo.toml")
    cargo.write_text(updated, encoding="utf-8")
    assert pinned(cargo) == revision
    return before, True


def run(cargo: Path, output: Path | None, fetch=read_api) -> tuple[str, bool]:
    revision = latest(fetch)
    before, updated = repin(cargo, revision)
    values = {
        "revision": revision,
        "previous_revision": before,
        "updated": str(updated).lower(),
    }
    if output:
        with output.open("a", encoding="utf-8") as file:
            file.write("".join(f"{key}={value}\n" for key, value in values.items()))
    print(f"Sunrise nightly SDK: {revision}")
    print(f"Project SDK pin: {before}")
    print("Updated SDK pin" if updated else "SDK already current")
    return revision, updated


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo", type=Path, default=ROOT / "Cargo.toml")
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args()
    try:
        run(args.cargo, args.github_output)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f"SDK synchronization failed: {error}\n")


if __name__ == "__main__":
    main()
)


def read_api(endpoint: str) -> dict:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "sunrise-qol-sdk-sync",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = Request(API + endpoint, headers=headers)
    try:
        with urlopen(request, timeout=25) as response:
            return json.load(response)
    except (HTTPError, URLError, ValueError) as error:
        raise ValueError(f"Could not verify Sunrise {endpoint}: {error}") from error


def latest(fetch=read_api) -> str:
    """Resolve the signed SDK for the exact commit of the official nightly."""
    nightly = fetch("/releases/tags/nightly")
    ref = fetch("/git/ref/tags/nightly")
    revision = nightly.get("target_commitish", "")
    if nightly.get("tag_name") != "nightly" or nightly.get("draft"):
        raise ValueError("No published Sunrise nightly")
    if not isinstance(revision, str) or not SHA.fullmatch(revision):
        raise ValueError("Sunrise nightly does not identify a full commit SHA")
    obj = ref.get("object", {})
    if obj.get("type") != "commit" or obj.get("sha") != revision:
        raise ValueError("Sunrise nightly release and tag point to different commits")

    sdk = fetch(f"/releases/tags/sdk-{revision}")
    if (sdk.get("tag_name") != f"sdk-{revision}" or sdk.get("draft")
            or sdk.get("target_commitish") != revision):
        raise ValueError(f"SDK release for nightly {revision} is not published")
    assets = {asset.get("name") for asset in sdk.get("assets", [])}
    required = {"sdk.json", "SHA256SUMS", "SHA256SUMS.sig",
                "api-x86_64-unknown-linux-gnu.tar.xz"}
    if not required.issubset(assets):
        raise ValueError(f"SDK {revision} is incomplete: missing {sorted(required - assets)}")
    return revision


def pinned(cargo: Path) -> str:
    data = tomllib.loads(cargo.read_text(encoding="utf-8"))
    revision = data["package"]["metadata"]["sunrise"]["sdk"]
    if not isinstance(revision, str) or not SHA.fullmatch(revision):
        raise ValueError("Cargo.toml has an invalid Sunrise SDK pin")
    return revision


def repin(cargo: Path, revision: str) -> tuple[str, bool]:
    if not SHA.fullmatch(revision):
        raise ValueError("Cannot pin an invalid Sunrise revision")
    before = pinned(cargo)
    if before == revision:
        return before, False
    original = cargo.read_text(encoding="utf-8")
    updated, count = SDK_LINE.subn(lambda match: match.group(1) + revision + match.group(2),
                                   original)
    if count != 1:
        raise ValueError("Expected one SDK pin in Cargo.toml")
    cargo.write_text(updated, encoding="utf-8")
    assert pinned(cargo) == revision
    return before, True


def run(cargo: Path, output: Path | None, fetch=read_api) -> tuple[str, bool]:
    revision = latest(fetch)
    before, updated = repin(cargo, revision)
    values = {
        "revision": revision,
        "previous_revision": before,
        "updated": str(updated).lower(),
    }
    if output:
        with output.open("a", encoding="utf-8") as file:
            file.write("".join(f"{key}={value}\n" for key, value in values.items()))
    print(f"Sunrise nightly SDK: {revision}")
    print(f"Project SDK pin: {before}")
    print("Updated SDK pin" if updated else "SDK already current")
    return revision, updated


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo", type=Path, default=ROOT / "Cargo.toml")
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args()
    try:
        run(args.cargo, args.github_output)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f"SDK synchronization failed: {error}\n")


if __name__ == "__main__":
    main()
