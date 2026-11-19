"""Command Help must paginate real per-player command names, not hardcoded names."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class CommandHelpTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/help.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_permissioned_command_cache(self):
        self.assertIn("event.names.iter().take(MAX_COMMANDS)", self.src)
        self.assertIn("map.insert(event.player, names)", self.src)
        self.assertIn("map.get(&event.player).cloned()", self.src)
        self.assertIn('if !on(host, "compact_help")', self.src)
        self.assertIn("if names.is_empty() { return Flow::Continue; }", self.src)
        self.assertIn("event.replies.push(styled(", self.src)
        self.assertIn("Flow::Cancel", self.src)
        self.assertIn("self.help.reset()", self.hooks)

    def test_pagination(self):
        self.assertIn("fn select_page(", self.src)
        self.assertIn("size).max(1))", self.src)
        self.assertIn('range("help_page_size", "Commands per Page", 8., 1., 50., 1.)', self.gui)
        self.assertIn('range("help_command_color", "Command Color", 2., 0., 15., 1.)', self.gui)
        self.assertIn('range("help_detail_color", "Detail Color", 7., 0., 15., 1.)', self.gui)
        self.assertIn('ModuleSpec { key: "compact_help", label: "Command Help"', self.gui)
        readme = (ROOT / "README.md").read_text()
        self.assertLess(readme.index("Glass bottles can temporarily"), readme.index("The help command gains clearer"))
        self.assertIn("- [x] The help command gains clearer", readme)
        self.assertIn("- [x] Active conduit effects suppress nearby", readme)

    def test_colored_clickable_commands_and_pages(self):
        self.assertIn("TextStyle {", self.src)
        self.assertIn("TextClick::SuggestCommand(format!", self.src)
        self.assertIn("TextClick::RunCommand(format!", self.src)
        self.assertIn('color(host, "help_command_color", 2.)', self.src)
        self.assertIn('color(host, "help_detail_color", 7.)', self.src)
        self.assertIn("COLORS: [&str; 16]", self.src)
        self.assertIn("if page > 1", self.src)
        self.assertIn("if page < total", self.src)

    def test_command_specific_usage_falls_back_to_vanilla(self):
        self.assertIn("argument.is_some_and(|value| value.parse::<usize>().is_err())", self.src)
        self.assertIn("return Flow::Continue;", self.src)
        self.assertNotIn("for command-specific usage, run", self.src)


if __name__ == "__main__":
    unittest.main()
