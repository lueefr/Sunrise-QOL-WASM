#!/usr/bin/env python3
"""Inspect WASM imports and exports without wasm-objdump or any Python dependency."""
from pathlib import Path
from zipfile import ZipFile
import argparse
import json


class Bytes:
    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0

    def take(self, size: int) -> bytes:
        if size < 0 or self.pos + size > len(self.data):
            raise ValueError("Truncated WASM binary")
        result = self.data[self.pos:self.pos + size]
        self.pos += size
        return result

    def u8(self) -> int:
        return self.take(1)[0]

    def uint(self) -> int:
        result = 0
        for i in range(10):
            part = self.u8()
            result |= (part & 0x7f) << (i * 7)
            if part < 128:
                return result
        raise ValueError("Invalid LEB128 integer")

    def name(self) -> str:
        return self.take(self.uint()).decode("utf-8", "replace")

    def limits(self):
        flags = self.uint()
        self.uint()
        if flags & 1:
            self.uint()
        if flags & 4:
            self.uint()

    def import_type(self, kind):
        if kind == 0:
            self.uint()
        elif kind == 1:
            self.u8()
            self.limits()
        elif kind == 2:
            self.limits()
        elif kind == 3:
            self.u8()
            self.u8()
        elif kind == 4:
            self.uint()
            self.uint()
        else:
            raise ValueError(f"Unsupported import kind: {kind}")


def symbols(data: bytes):
    stream = Bytes(data)
    if stream.take(8) != b"\0asm\x01\0\0\0":
        raise ValueError("Not a WebAssembly v1 module")
    imports = []
    exports = []
    while stream.pos < len(data):
        section = stream.u8()
        length = stream.uint()
        inner = Bytes(stream.take(length))
        if section == 2:
            for _ in range(inner.uint()):
                module, name, kind = inner.name(), inner.name(), inner.u8()
                imports.append({"module": module, "symbol": name, "kind": kind})
                inner.import_type(kind)
        elif section == 7:
            for _ in range(inner.uint()):
                name, kind = inner.name(), inner.u8()
                inner.uint()
                exports.append({"symbol": name, "kind": kind})
    return imports, exports


def contents(path: Path, member: str = "mod.wasm") -> bytes:
    if path.suffix == ".smod" or path.suffix == ".zip":
        with ZipFile(path) as archive:
            return archive.read(member)
    return path.read_bytes()


def main():
    parser = argparse.ArgumentParser(description="Inspect unresolved imports of mod.wasm")
    parser.add_argument("package", type=Path, help="A .smod or .wasm file")
    parser.add_argument("--std", type=Path, help="sunrise-wasm-std.wasm shipped with the game")
    parser.add_argument("--json", action="store_true", help="Print results as JSON")
    opts = parser.parse_args()
    imports, exports = symbols(contents(opts.package))
    std_exports = []
    if opts.std:
        _, std_exports = symbols(contents(opts.std))
    std_names = {(s['symbol'], s['kind']) for s in std_exports}
    external = [i for i in imports if i['module'] == 'env' and i['symbol'].startswith('_R')]
    missing = [i['symbol'] for i in external if (i['symbol'], i['kind']) not in std_names]
    result = {
        'module': str(opts.package),
        'std': str(opts.std) if opts.std else None,
        'imports': len(imports),
        'exports': len(exports),
        'rust_symbols_imported': len(external),
        'rust_symbols_not_exported_by_std': missing if opts.std else None,
    }
    if opts.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        for key, value in result.items():
            if isinstance(value, list):
                print(f'{key}: {len(value)}')
                for v in value[:100]:
                    print('  ', v)
            else:
                print(f'{key}: {value}')
        if opts.std and missing:
            print('Warning: some imports may be provided by other group members.')
            print('Compare this list against all loaded members, not just the standard library.')


if __name__ == '__main__':
    main()
