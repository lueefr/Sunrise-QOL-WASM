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
        for before, after in (
            ((1, 0, 0), (1, 0, 1)),
            ((1, 0, 8), (1, 0, 9)),
            ((1, 0, 9), (1, 1, 0)),
            ((1, 1, 0), (1, 1, 1)),
            ((1, 9, 8), (1, 9, 9)),
            ((1, 9, 9), (2, 0, 0)),
            ((2, 0, 0), (2, 0, 1)),
            ((2, 9, 9), (3, 0, 0)),
        ):
            with self.subTest(before=before):
                self.assertEqual(release.next_version(before, [("fix: repair", "")]), after)
                self.assertEqual(release.next_version(before, [("feat: feature", "")]), after)
                self.assertEqual(release.next_version(before, [("chore: update", "")]), after)
        with self.assertRaises(ValueError):
            release.next_version((1, 10, 0))
        with self.assertRaises(ValueError):
            release.next_version((2, 1, 10))

    def test_legacy_tag_is_ignored_by_exact_object(self):
        original_root = release.ROOT
        original_object = release.LEGACY_TAG_OBJECT
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
                git("commit", "-qm", "initial release")
                git("tag", "v1.0.0")
                (target / "src/lib.rs").write_text("fn modified() {}\n")
                git("add", "src/lib.rs")
                git("commit", "-qm", "feat: future release")
                git("tag", "-a", "v2.0.0", "-m", "Incorrect old numbering")
                release.LEGACY_TAG_OBJECT = git("rev-parse", "refs/tags/v2.0.0")
                self.assertEqual(release.latest(), ("v1.0.0", (1, 0, 0)))
                release.LEGACY_TAG_OBJECT = "0" * 40
                self.assertEqual(release.latest(), ("v2.0.0", (2, 0, 0)))
            finally:
                release.LEGACY_TAG_OBJECT = original_object
                release.ROOT = original_root

    def test_commit_classification(self):
        self.assertEqual(release.kind("feat(gui): scrolling"), "major")
        self.assertEqual(release.kind("fix(unlimited): cost"), "minor")
        self.assertEqual(release.kind("refactor: split module"), "patch")
        self.assertEqual(release.kind("chore: cleanup", "BREAKING CHANGE: ABI"), "major")

    def test_sandbox_and_tests_trigger_release(self):
        root = release.ROOT
        try:
            for changed in (
                "tests/test_sandbox.py",
                "tools/sandbox/sunrise_qol_smoke.rs.txt",
                ".github/workflows/sandbox.yml",
            ):
                with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                    target = Path(directory)
                    release.ROOT = target

                    def git(*arguments):
                        return subprocess.run(["git", *arguments], cwd=target,
                                              check=True, capture_output=True,
                                              text=True).stdout.strip()

                    git("init", "-q")
                    git("config", "user.name", "Release Tests")
                    git("config", "user.email", "test@example.invalid")
                    (target / "src").mkdir()
                    (target / "src/lib.rs").write_text("fn original() {}\\n")
                    git("add", "src/lib.rs")
                    git("commit", "-qm", "initial: import mod")
                    git("tag", "v1.1.1")

                    path = target / changed
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("updated verification\\n")
                    git("add", changed)
                    git("commit", "-qm", "test(sandbox): add behavioral assertion")

                    self.assertEqual(release.changed_paths("v1.1.1"), [changed])
                    self.assertEqual(release.latest(), ("v1.1.1", (1, 1, 1)))
                    self.assertEqual(release.next_version((1, 1, 1)), (1, 1, 2))
        finally:
            release.ROOT = root

    def test_uncommitted_sdk_pin_starts_release(self):
        root = release.ROOT
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            release.ROOT = target

            def git(*args):
                return subprocess.run(["git", *args], cwd=target,
                                      check=True, capture_output=True,
                                      text=True).stdout.strip()
            try:
                git("init", "-q")
                git("config", "user.name", "Release Tests")
                git("config", "user.email", "test@example.invalid")
                cargo = target / "Cargo.toml"
                cargo.write_text('[package]\nversion = "1.1.2"\n'
                                 '[package.metadata.sunrise]\nsdk = "old"\n')
                git("add", "Cargo.toml")
                git("commit", "-qm", "chore(release): v1.1.2")
                git("tag", "v1.1.2")
                self.assertEqual(release.changed_paths("v1.1.2"), [])
                cargo.write_text(cargo.read_text().replace('"old"', '"new"'))
                self.assertEqual(release.changed_paths("v1.1.2"), ["Cargo.toml"])
                self.assertEqual(release.next_version((1, 1, 2)), (1, 1, 3))
                git("add", "Cargo.toml")
                self.assertEqual(release.changed_paths("v1.1.2"), ["Cargo.toml"])
            finally:
                release.ROOT = root

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
                                 (1, 0, 1))
            finally:
                release.ROOT = root


if __name__ == "__main__":
    unittest.main()
