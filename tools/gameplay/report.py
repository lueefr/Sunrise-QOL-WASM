#!/usr/bin/env python3
"""Fail closed: attest only observations from actual Sunrise Game world scenarios."""
import argparse
import json
import os
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVENT = re.compile(r"^QOL_GAMEPLAY\|([a-z0-9_.-]+)\|(PASS|FAIL)$", re.M)
MODULE = re.compile(r'ModuleSpec \{ key: "([^"]+)"')


def validate(log: str, required: list[str], code: int):
    observed = {}
    for key, status in EVENT.findall(log):
        observed.setdefault(key, []).append(status)
    expected = set(required)
    seen = set(observed)
    passed = (code == 0 and expected == seen
              and all(observed[key] == ["PASS"] for key in required)
              and "test result: ok." in log and "1 passed" in log)
    return passed, {
        "required": required,
        "observed": {key: values for key, values in sorted(observed.items())},
        "missing": sorted(expected - seen),
        "unexpected": sorted(seen - expected),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--exit-code", type=int, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--package", type=Path, required=True)
    args = parser.parse_args()
    ledger = json.loads((Path(__file__).with_name("cases.json")).read_text())
    modules = MODULE.findall((ROOT / "src/gui.rs").read_text())
    if len(modules) != len(set(modules)):
        raise ValueError("Duplicate module IDs")
    if ledger["module"] not in modules:
        raise ValueError("A scenario targets a missing module")
    with (ROOT / "Cargo.toml").open("rb") as file:
        sdk = tomllib.load(file)["package"]["metadata"]["sunrise"]["sdk"]
    if sdk != args.source:
        raise ValueError("Game revision differs from the pinned SDK")
    log = args.log.read_text(errors="replace") if args.log.exists() else ""
    passed, scenarios = validate(log, ledger["required"], args.exit_code)
    proven = [ledger["module"]] if passed else []
    evidence = {
        "schema": 1,
        "kind": "real-game-state-observations",
        "runtime": ledger["engine"],
        "sdk_revision": args.source,
        "package": args.package.name,
        "test_exit_code": args.exit_code,
        "gate_passed": passed,
        "scenarios": scenarios,
        "modules_total": len(modules),
        "modules_with_real_game_tests": proven,
        "modules_without_real_game_tests": sorted(set(modules) - set(proven)),
        "modules_certified_fully_one_to_one": [],
        "warning": "Passing representative real-game scenarios does not certify complete 1:1 parity.",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(evidence, indent=2, ensure_ascii=False) + "\n")
    result = "PASS" if passed else "FAIL"
    print(f"{result} Sunrise Game: {len(scenarios['observed'])}/{len(ledger['required'])} "
          f"observations, {len(proven)}/{len(modules)} modules with real-engine tests")
    if not passed:
        print(f"Missing: {scenarios['missing']}; unexpected: {scenarios['unexpected']}", file=sys.stderr)
    if os.getenv("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
            summary.write(f"### Real Sunrise gameplay gate: {result}\n\n")
            summary.write(f"- SDK revision: {args.source}\n")
            summary.write(f"- World-state cases: {len(scenarios['observed'])}/{len(ledger['required'])}\n")
            summary.write(f"- Modules with real game tests: {len(proven)}/{len(modules)}\n")
            summary.write("- Modules certified fully 1:1: 0 (other branches untested)\n")
            summary.write("- Details: gameplay-evidence.json and gameplay-run.log artifacts\n")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
