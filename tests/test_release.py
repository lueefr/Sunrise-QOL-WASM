"""Offline version-policy regression tests."""
import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "tools/release.py"
spec = importlib.util.spec_from_file_location("release_policy", SCRIPT)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleasePolicyTests(unittest.TestCase):
    def test_version_examples(self):
        self.assertEqual(release.next_version(None, [("Initial commit", "")]), (1, 0, 0))
        self.assertEqual(release.next_version((1, 0, 0), [("fix: mend enchantment", "")]), (1, 1, 1))
        self.assertEqual(release.next_version((1, 1, 1), [("fix: second bug", "")]), (1, 2, 2))
        self.assertEqual(release.next_version((1, 1, 1), [("feat: new module", "")]), (2, 0, 0))
        self.assertEqual(release.next_version((2, 0, 0), [("refactor: cleanup", "")]), (2, 0, 1))
        self.assertEqual(release.next_version((2, 0, 1), [("fix: bug", "")]), (2, 1, 2))
        self.assertEqual(release.next_version((1, 0, 0), [
            ("fix: bug", ""), ("feat: another module", "")]), (2, 0, 0))

    def test_commit_classification(self):
        self.assertEqual(release.kind("feat(gui): scrolling"), "major")
        self.assertEqual(release.kind("fix(unlimited): cost"), "minor")
        self.assertEqual(release.kind("refactor: split module"), "patch")
        self.assertEqual(release.kind("chore: cleanup", "BREAKING CHANGE: ABI"), "major")

    def test_documentation_only_does_not_count(self):
        root = release.ROOT
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            release.ROOT = target

            def git(*arguments):
                return subprocess.run(["git", *arguments], cwd=target,
                                      check=True, capture_output=True, text=True).stdout.strip()

            try:
                git("init", "-q")
                git("config", "user.name", "Release Tests")
                git("config", "user.email", "test@example.invalid")
                (target / "src").mkdir()
                (target / "src/lib.rs").write_text("fn original() {}\n")
                git("add", "src/lib.rs")
                git("commit", "-qm", "initial: import mod")
                git("tag", "v1.0.0")
                (target / "README.md").write_text("documentation only\n")
                git("add", "README.md")
                git("commit", "-qm", "docs: explain usage")
                self.assertEqual(release.changed_paths("v1.0.0"), [])
                (target / "src/lib.rs").write_text("fn change() {}\n")
                git("add", "src/lib.rs")
                git("commit", "-qm", "fix: issue 123")
                self.assertEqual(release.changed_paths("v1.0.0"), ["src/lib.rs"])
                self.assertEqual(release.next_version((1, 0, 0), release.messages("v1.0.0")),
                                 (1, 1, 1))
            finally:
                release.ROOT = root


if __name__ == "__main__":
    unittest.main()
