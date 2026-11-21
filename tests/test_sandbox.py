"""Real binary compatibility checks for the headless Sunrise sandbox."""
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "sandbox"))
from check import verify, verify_harvest_tags, HARVEST_TAGS


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

    def test_original_harvest_tags_are_in_the_package(self):
        prefix = "data/rightclickharvest/tags/"
        with ZipFile(self.package, "w") as archive:
            archive.writestr("mod.wasm", exports("_Rtag"))
            for name in HARVEST_TAGS:
                path = ROOT / (prefix + name)
                archive.write(path, prefix + name)
        report = verify_harvest_tags(self.package)
        self.assertTrue(report["harvest_tags_passed"], report)
        self.assertEqual(report["harvest_tag_files"], 6)
        with ZipFile(self.package, "a") as archive:
            archive.writestr(prefix + "item/low_tier_hoes.json", b'{}')
        # A duplicate zip member or modified source must not pass silently.
        changed = verify_harvest_tags(self.package)
        self.assertFalse(changed["harvest_tags_passed"])
        self.assertIn(prefix + "item/low_tier_hoes.json", changed["modified_harvest_tags"])

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

    def test_smoke_runs_the_actual_loader_without_gameplay(self):
        source = (ROOT / "tools/sandbox/sunrise_qol_smoke.rs.txt").read_text()
        self.assertIn("sunrise_mod_wasm::{WasmPackage, load}", source)
        self.assertIn("Package::read(path)", source)
        self.assertIn("loaded.initialize(&[info])", source)
        self.assertIn("assert!(loaded.refused.is_empty()", source)
        self.assertNotIn("dispatch_", source)
        self.assertNotIn("check_help(", source)

    def test_release_requires_the_shared_loader_gate(self):
        release = (ROOT / ".github/workflows/release.yml").read_text()
        sandbox = (ROOT / ".github/workflows/sandbox.yml").read_text()
        shared = (ROOT / ".github/actions/validate/action.yml").read_text()
        self.assertIn("uses: ./.github/actions/validate", release)
        self.assertIn("uses: ./.github/actions/validate", sandbox)
        self.assertIn("bash tools/sandbox/run.sh", shared)
        self.assertIn("tools/sandbox/check.py", shared)
        self.assertNotIn("tools/gameplay/run.sh", release)
        self.assertNotIn("tools/gameplay/run.sh", sandbox)
        self.assertNotIn("tools/gameplay/run.sh", shared)
        self.assertLess(release.index("Compile and verify the mod in Sunrise"),
                        release.index("Commit version and tag only after a successful build"))


if __name__ == "__main__":
    unittest.main()
