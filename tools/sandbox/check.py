#!/usr/bin/env python3
"""Check the built guest against the shared standard library before Wasmtime loads it."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from wasm_link_check import contents, symbols


def verify(package: Path, standard: Path) -> dict:
    imports, guest_exports = symbols(contents(package))
    _, std_exports = symbols(contents(standard))
    known = {(entry["symbol"], entry["kind"]) for entry in guest_exports + std_exports}
    rust_imports = [
        entry for entry in imports
        if entry["module"] == "env" and entry["symbol"].startswith("_R")
    ]
    missing = sorted({
        entry["symbol"] for entry in rust_imports
        if (entry["symbol"], entry["kind"]) not in known
    })
    return {
        "package": str(package),
        "standard": str(standard),
        "rust_imports": len(rust_imports),
        "standard_exports": len(std_exports),
        "missing_rust_symbols": missing,
        "passed": len(missing) == 0,
    }


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("Usage: python tools/sandbox/check.py package.smod sunrise-wasm-std.wasm")
    result = verify(Path(sys.argv[1]), Path(sys.argv[2]))
    print(json.dumps(result, indent=2))
    if not result["passed"]:
        sys.exit("Unresolved Rust symbols: the .smod is incompatible with this std member")


if __name__ == "__main__":
    main()
