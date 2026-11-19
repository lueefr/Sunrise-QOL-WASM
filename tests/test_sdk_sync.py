"""Regression checks for selecting the published Sunrise nightly SDK."""
import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "tools/sdk_sync.py"
spec = importlib.util.spec_from_file_location("sdk_sync", SCRIPT)
sdk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sdk)

OLD = "2d8126336e579777eabf9b2fd8b3d476236a04f2"
NEW = "520fc4882de3abb8ecd8b98293f4d7263bc88181"


def release_api(path):
    if path == "/releases/tags/nightly":
        return {"tag_name": "nightly", "target_commitish": NEW, "draft": False}
    if path == "/git/ref/tags/nightly":
        return {"object": {"type": "commit", "sha": NEW}}
    if path == f"/releases/tags/sdk-{NEW}":
        return {
            "tag_name": f"sdk-{NEW}", "target_commitish": NEW, "draft": False,
            "assets": [{"name": name} for name in (
                "sdk.json", "SHA256SUMS", "SHA256SUMS.sig",
                "api-x86_64-unknown-linux-gnu.tar.xz",
            )],
        }
    raise AssertionError(f"Unexpected API call: {path}")


class SdkSyncTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.cargo = Path(self.tmp.name) / "Cargo.toml"
        self.cargo.write_text(
            '[package]\nname = "test"\nversion = "1.1.2"\n'
            f'\n[package.metadata.sunrise]\nsdk = "{OLD}"\n'
            '\n[dependencies]\nserde_json = "1"\n'
        )

    def test_published_nightly_repin(self):
        output = Path(self.tmp.name) / "output"
        rev, updated = sdk.run(self.cargo, output, release_api)
        self.assertEqual((rev, updated), (NEW, True))
        self.assertEqual(sdk.pinned(self.cargo), NEW)
        self.assertIn("previous_revision=" + OLD, output.read_text())
        self.assertIn("updated=true\n", output.read_text())
        self.assertIn('[dependencies]\nserde_json = "1"', self.cargo.read_text())
        self.assertIn('version = "1.1.2"', self.cargo.read_text())

    def test_unchanged_nightly_is_idempotent(self):
        sdk.run(self.cargo, None, release_api)
        first = self.cargo.read_bytes()
        output = Path(self.tmp.name) / "output"
        rev, updated = sdk.run(self.cargo, output, release_api)
        self.assertEqual((rev, updated), (NEW, False))
        self.assertEqual(self.cargo.read_bytes(), first)
        self.assertIn("updated=false\n", output.read_text())

    def test_rejects_unpublished_sdk_without_writing(self):
        original = self.cargo.read_bytes()

        def unavailable(path):
            if path == f"/releases/tags/sdk-{NEW}":
                return {"tag_name": f"sdk-{NEW}", "draft": True,
                        "target_commitish": NEW, "assets": []}
            return release_api(path)

        with self.assertRaisesRegex(ValueError, "not published"):
            sdk.run(self.cargo, None, unavailable)
        self.assertEqual(self.cargo.read_bytes(), original)

    def test_rejects_tag_race_without_writing(self):
        original = self.cargo.read_bytes()

        def changed_tag(path):
            if path == "/git/ref/tags/nightly":
                return {"object": {"type": "commit", "sha": OLD}}
            return release_api(path)

        with self.assertRaisesRegex(ValueError, "different commits"):
            sdk.run(self.cargo, None, changed_tag)
        self.assertEqual(self.cargo.read_bytes(), original)

    def test_rejects_missing_sdk_assets(self):
        def missing(path):
            result = release_api(path)
            if path == f"/releases/tags/sdk-{NEW}":
                result["assets"] = [{"name": "sdk.json"}]
            return result

        with self.assertRaisesRegex(ValueError, "incomplete"):
            sdk.run(self.cargo, None, missing)
        self.assertEqual(sdk.pinned(self.cargo), OLD)

    def test_rejects_invalid_existing_pin(self):
        self.cargo.write_text(self.cargo.read_text().replace(OLD, "main"))
        with self.assertRaisesRegex(ValueError, "invalid Sunrise SDK pin"):
            sdk.run(self.cargo, None, release_api)


if __name__ == "__main__":
    unittest.main()
