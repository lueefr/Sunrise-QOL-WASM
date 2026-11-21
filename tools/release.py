#!/usr/bin/env python3
"""Plan sequential Sunrise QOL releases using decimal version digits."""
from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATTERN = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
# The first v2.0.0 was incorrectly numbered and must be replaced by v1.0.1.
# Match its annotated-tag object so a future legitimate v2.0.0 is unaffected.
LEGACY_TAG = "v2.0.0"
LEGACY_TAG_OBJECT = "3f51c263fd5f4743e4a7c4798e28cf7cbae3eeb8"
# Treat changes to the real-runtime verification as releasable work: the
# validated package and its release gate can change without touching src/.
MOD_PATHS = (
    ".github/workflows/release.yml", ".github/workflows/sandbox.yml",
    "tools/release.py", ".github/actions/validate", "src", "assets", "data",
    "licenses", "tools/parity", "Cargo.toml", "sunrise-mod.toml",
    "instalar.fish", "preparar-rust.fish",
    "diagnosticar.fish", "tools/wasm_link_check.py",
    "tools/migrate_settings.py", "tools/sdk_sync.py", "tools/sdk_cache_prune.py",
    "tools/install_sdk_tool.sh", "tools/sandbox", "tests",
)


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def latest() -> tuple[str | None, tuple[int, int, int] | None]:
    versions = []
    for tag in git("tag", "--list", "v*").splitlines():
        match = PATTERN.fullmatch(tag)
        if not match:
            continue
        if tag == LEGACY_TAG and git("rev-parse", f"refs/tags/{tag}") == LEGACY_TAG_OBJECT:
            continue
        version = tuple(int(part) for part in match.groups())
        if version[0] >= 1 and 0 <= version[1] <= 9 and 0 <= version[2] <= 9:
            versions.append((version, tag))
    if not versions:
        return None, None
    version, tag = max(versions)
    return tag, version


def changed_paths(previous: str | None) -> list[str]:
    if previous is None:
        committed = git("ls-files", "--", *MOD_PATHS).splitlines()
    else:
        committed = git("diff", "--name-only", f"{previous}..HEAD",
                        "--", *MOD_PATHS).splitlines()
    # CI moves the SDK pin in the working tree before the release is planned.
    # The pin is committed only *after* the new SDK builds and passes the sandbox.
    working = git("diff", "--name-only", "--", *MOD_PATHS).splitlines()
    staged = git("diff", "--cached", "--name-only", "--", *MOD_PATHS).splitlines()
    return sorted(set(committed + working + staged))


def messages(previous: str | None) -> list[tuple[str, str]]:
    revision = f"{previous}..HEAD" if previous else "HEAD"
    # NUL preserves commit messages with multiple lines.
    data = subprocess.check_output(
        ["git", "log", "--no-merges", "--format=%s%x1f%b%x00", revision,
         "--", *MOD_PATHS], cwd=ROOT
    )
    result = []
    for block in data.decode("utf-8", "replace").split("\x00"):
        if "\x1f" in block:
            subject, body = block.split("\x1f", 1)
            result.append((subject.strip(), body.strip()))
    return result


def kind(subject: str, body: str = "") -> str:
    if re.match(r"^(feat|feature)(\([^)]+\))?!:", subject, re.I) or re.search(
        r"(?m)^BREAKING(?:-| )CHANGE:", body
    ):
        return "major"
    if re.match(r"^(feat|feature)(\([^)]+\))?:", subject, re.I):
        return "major"
    if re.match(r"^(fix|bugfix|bug)(\([^)]+\))?:", subject, re.I):
        return "minor"
    return "patch"


def next_version(previous: tuple[int, int, int] | None,
                 commits: list[tuple[str, str]] | None = None) -> tuple[int, int, int]:
    """Increment once per release, regardless of how commits are classified."""
    if previous is None:
        return (1, 0, 0)
    major, minor, patch = previous
    if major < 1 or not (0 <= minor <= 9 and 0 <= patch <= 9):
        raise ValueError("version must use one decimal digit for minor and patch")
    patch += 1
    if patch == 10:
        patch = 0
        minor += 1
    if minor == 10:
        minor = 0
        major += 1
    return (major, minor, patch)

def plan(output: Path | None) -> None:
    previous_tag, previous = latest()
    paths = changed_paths(previous_tag)
    current = next_version(previous)
    version = ".".join(map(str, current))
    values = {
        "changed": "true" if paths else "false",
        "version": version,
        "tag": f"v{version}",
        "previous_tag": previous_tag or "",
        "files_changed": str(len(paths)),
    }
    text = "".join(f"{key}={value}\n" for key, value in values.items())
    if output:
        with output.open("a", encoding="utf-8") as stream:
            stream.write(text)
    print(text, end="")


def stamp(version: str) -> None:
    if not re.fullmatch(r"[0-9]+\.[0-9]\.[0-9]", version) or int(version.split(".")[0]) < 1:
        raise ValueError("invalid sequential version")
    for name in ("Cargo.toml", "sunrise-mod.toml"):
        path = ROOT / name
        content = path.read_text(encoding="utf-8")
        updated, count = re.subn(r'(?m)^(version\s*=\s*)"\d+\.\d+\.\d+"',
                                 rf'\g<1>"{version}"', content, count=1)
        if count != 1:
            raise ValueError(f"missing version in {name}")
        path.write_text(updated, encoding="utf-8")
    (ROOT / "VERSION").write_text(version + "\n", encoding="utf-8")


def notes(path: Path, version: str) -> None:
    previous, _ = latest()
    commits = messages(previous)
    sections = {"major": [], "minor": [], "patch": []}
    for subject, body in commits:
        if subject and not subject.startswith("chore(release):"):
            sections[kind(subject, body)].append(subject)
    lines = [f"# Sunrise QOL v{version}", "", "Compiled Sunrise WASM mod (`.smod`).", ""]
    if previous:
        lines.append(f"Changes since {previous} (source and verification files):")
    else:
        lines.append("Initial release of the existing Sunrise QOL source tree.")
    lines.append("")
    for label, key in (("Features / breaking changes", "major"),
                       ("Bug fixes", "minor"), ("Maintenance", "patch")):
        if sections[key]:
            lines.extend([f"## {label}", ""])
            lines.extend(f"- {subject}" for subject in reversed(sections[key]))
            lines.append("")
    lines.extend([
        "The package is compiled automatically; in-game feature parity is not implied.",
        "Enable it in Sunrise only after checking compatibility with your game build.", "",
    ])
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action", required=True)
    p = sub.add_parser("plan")
    p.add_argument("--github-output", type=Path)
    s = sub.add_parser("stamp")
    s.add_argument("version")
    n = sub.add_parser("notes")
    n.add_argument("version")
    n.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.action == "plan":
        plan(args.github_output)
    elif args.action == "stamp":
        stamp(args.version)
    else:
        notes(args.output, args.version)


if __name__ == "__main__":
    main()
