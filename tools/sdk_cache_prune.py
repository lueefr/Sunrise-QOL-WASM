#!/usr/bin/env python3
"""Evict old nightly-specific Sunrise caches after a successful main release."""
from __future__ import annotations

import json
import os
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

SHA = re.compile(r"[0-9a-f]{40}\Z")
KINDS = ("cli", "sdk", "std", "runtime")


def outdated(caches: list[dict], revision: str, platform: str) -> list[int]:
    prefix = platform + "-sunrise-"
    result = []
    for entry in caches:
        key = entry.get("key", "")
        if not key.startswith(prefix):
            continue
        if not any(key.startswith(prefix + kind + "-") for kind in KINDS):
            continue
        if f"-{revision}-" in key:
            continue
        result.append(entry["id"])
    return result


def api(path: str, method: str = "GET") -> dict:
    token = os.environ["GH_TOKEN"]
    repository = os.environ["GITHUB_REPOSITORY"]
    request = Request(
        f"https://api.github.com/repos/{repository}/actions/caches{path}",
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "sunrise-qol-cache-prune",
        },
        method=method,
    )
    with urlopen(request, timeout=30) as response:
        if method == "DELETE":
            return {}
        return json.load(response)


def main() -> None:
    if len(sys.argv) != 2 or not SHA.fullmatch(sys.argv[1]):
        sys.exit("Usage: sdk_cache_prune.py NIGHTLY_SHA")
    revision = sys.argv[1]
    platform = os.environ.get("RUNNER_OS", "Linux") + "-" + os.environ.get("RUNNER_ARCH", "X64")
    caches = []
    page = 1
    while True:
        batch = api(f"?per_page=100&page={page}").get("actions_caches", [])
        caches.extend(batch)
        if len(batch) < 100:
            break
        page += 1
    old = outdated(caches, revision, platform)
    for cache_id in old:
        try:
            api(f"/{cache_id}", "DELETE")
            print(f"Removed old Sunrise cache {cache_id}")
        except HTTPError as error:
            if error.code != 404:
                raise
    print(f"Sunrise SDK {revision}: retained current caches, removed {len(old)} older entries")


if __name__ == "__main__":
    try:
        main()
    except (HTTPError, URLError, KeyError, ValueError) as error:
        sys.exit(f"Could not prune Sunrise caches: {error}")
