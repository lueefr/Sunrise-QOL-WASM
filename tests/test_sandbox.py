"""Real binary compatibility checks for the headless Sunrise sandbox."""
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "sandbox"))
from check import verify


def integer(number):
    result = bytearray()
    while True:
        item = number & 0x7F
        number >>= 7
        result.append(item | 0x80 if number else item)
        if not number:
            return bytes(result)


def name(value):
    encoded = value.encode()
    return integer(len(encoded)) + encoded


def section(section_id, body):
    return bytes([section_id]) + integer(len(body)) + body


def imports(symbol):
    return b"\0asm\x01\0\0\0" + section(
        2, integer(1) + name("env") + name(symbol) + b"\0" + integer(0)
    )


def exports(symbol):
    return b"\0asm\x01\0\0\0" + section(
        7, integer(1) + name(symbol) + b"\0" + integer(0)
    )


class SandboxTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.package = self.root / "mod.smod"
        self.std = self.root / "std.wasm"

    def guest(self, source):
        with ZipFile(self.package, "w") as archive:
            archive.writestr("mod.wasm", source)

    def test_matching_rust_symbol(self):
        self.guest(imports("_Rexample"))
        self.std.write_bytes(exports("_Rexample"))
        report = verify(self.package, self.std)
        self.assertTrue(report["passed"])
        self.assertEqual(report["rust_imports"], 1)
        self.assertEqual(report["missing_rust_symbols"], [])

    def test_rejects_missing_symbol(self):
        self.guest(imports("_Rmissing"))
        self.std.write_bytes(exports("_Rother"))
        report = verify(self.package, self.std)
        self.assertFalse(report["passed"])
        self.assertEqual(report["missing_rust_symbols"], ["_Rmissing"])

    def test_rejects_corrupt_binary(self):
        self.guest(b"not wasm")
        self.std.write_bytes(exports("_Rfoo"))
        with self.assertRaises(ValueError):
            verify(self.package, self.std)

    def test_release_gates_on_real_loader(self):
        workflow = (ROOT / ".github/workflows/release.yml").read_text()
        a = workflow.index("name: Validate mod package")
        b = workflow.index("name: Smoke test with actual Sunrise runtime")
        c = workflow.index("name: Commit version and tag only after a successful build")
        self.assertLess(a, b)
        self.assertLess(b, c)
        self.assertIn("tools/sandbox/run.sh", workflow)
        self.assertIn("tools/sandbox/check.py", workflow)


if __name__ == "__main__":
    unittest.main()
