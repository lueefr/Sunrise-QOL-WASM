#!/usr/bin/env python3
"""Plan Sunrise QOL releases from mod changes and Conventional Commits."""
from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATTERN = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
MOD_PATHS = (
    "src", "assets", "Cargo.toml", "sunrise-mod.toml", "instalar.fish",
    "preparar-rust.fish", "diagnosticar.fish", "tools/wasm_link_check.py",
    "tools/migrate_settings.py",
)


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def latest() -> tuple[str | None, tuple[int, int, int] | None]:
    versions = []
    for tag in git("tag", "--list", "v*").splitlines():
        match = PATTERN.fullmatch(tag)
        if match:
            versions.append((tuple(int(part) for part in match.groups()), tag))
    if not versions:
        return None, None
    version, tag = max(versions)
    return tag, version


def changed_paths(previous: str | None) -> list[str]:
    if previous is None:
        return git("ls-files", "--", *MOD_PATHS).splitlines()
    return git("diff", "--name-only", f"{previous}..HEAD", "--", *MOD_PATHS).splitlines()


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
                 commits: list[tuple[str, str]]) -> tuple[int, int, int]:
    if previous is None:
        return (1, 0, 0)
    major, minor, count = previous
    changes = {kind(subject, body) for subject, body in commits}
    if "major" in changes:
        return major + 1, 0, 0
    if "minor" in changes:
        return major, minor + 1, count + 1
    # The third component counts updates inside a major version.
    # With no Conventional Commit classification, this is a maintenance update.
    return major, minor, count + 1


def plan(output: Path | None) -> None:
    previous_tag, previous = latest()
    paths = changed_paths(previous_tag)
    commits = messages(previous_tag) if paths else []
    current = next_version(previous, commits)
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
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError("invalid version")
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
    lines = [f"# Sunrise QOL v{version}", "", "Compiled Sunrise WASM mod (\`.smod\`).", ""]
    if previous:
        lines.append(f"Changes since {previous} (mod files only):")
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
