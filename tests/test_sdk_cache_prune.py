"""Check that cache rotation keeps the current published SDK revision."""
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "tools/sdk_cache_prune.py"
spec = importlib.util.spec_from_file_location("sdk_cache_prune", SCRIPT)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

OLD = "a" * 40
NEW = "b" * 40


class CachePruneTests(unittest.TestCase):
    def test_removes_old_sdk_and_related_artifacts(self):
        caches = [
            {"id": 1, "key": f"Linux-X64-sunrise-sdk-{OLD}-v1"},
            {"id": 2, "key": f"Linux-X64-sunrise-cli-{OLD}-v1"},
            {"id": 3, "key": f"Linux-X64-sunrise-runtime-{OLD}-v1"},
            {"id": 4, "key": f"Linux-X64-sunrise-std-{OLD}-v1"},
            {"id": 5, "key": f"Linux-X64-sunrise-sdk-{NEW}-v2"},
            {"id": 6, "key": f"Linux-X64-sunrise-runtime-{NEW}-v2"},
            {"id": 7, "key": "Linux-X64-not-our-cache"},
            {"id": 8, "key": f"Windows-X64-sunrise-sdk-{OLD}-v1"},
        ]
        self.assertEqual(module.outdated(caches, NEW, "Linux-X64"), [1, 2, 3, 4])

    def test_no_update_keeps_everything(self):
        caches = [{"id": 5, "key": f"Linux-X64-sunrise-sdk-{NEW}-v2"}]
        self.assertEqual(module.outdated(caches, NEW, "Linux-X64"), [])


if __name__ == "__main__":
    unittest.main()
