from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class NumericFieldTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gui = (ROOT / "src/gui.rs").read_text()

    def test_ranges_use_native_edit_box(self):
        self.assertIn("TextField::new(rect, 16)", self.gui)
        self.assertIn("field.set_value(&format!(\"{value:.0}\"))", self.gui)
        self.assertIn("self.widgets.add_text_field(field)", self.gui)
        self.assertIn("self.number([x + w - 88., next, 56., 20.], *opt, value, active)", self.gui)
        self.assertIn('self.button([x + w - 114., next, 23., 20.], "-"', self.gui)
        self.assertIn('self.button([x + w - 29., next, 23., 20.], "+"', self.gui)
        self.assertIn('opt.key == "persistentLeavesBehavior"', self.gui)

    def test_edits_are_not_saved_per_character(self):
        self.assertIn("Outcome::Changed(id) => screen.mark_number(id)", self.gui)
        self.assertIn("Outcome::Submitted(id)", self.gui)
        self.assertIn("screen.commit_number(host, id)", self.gui)
        self.assertIn("if before != screen.widgets.focused()", self.gui)
        self.assertIn("screen.widgets.set_focus(host, None)", self.gui)
        self.assertIn("fn valid_number(value: &str, min: f64, max: f64)", self.gui)
        self.assertIn("number.is_finite() && number >= min && number <= max", self.gui)
        self.assertIn("value.round().clamp(min, max)", self.gui)
        self.assertIn("save(host, option.key, &format!(\"{bounded:.0}\"))", self.gui)

    def test_pending_input_is_handled_before_rebuild_or_close(self):
        self.assertGreaterEqual(self.gui.count("screen.commit_focused(host)"), 5)
        self.assertIn("let before = screen.widgets.focused()", self.gui)
        self.assertIn('event.key == "key.keyboard.escape"', self.gui)
        self.assertIn('self.overlay.store(false, Ordering::Relaxed)', self.gui)
        self.assertIn("fn wheel(&self,", self.gui)
        self.assertIn("fn pressed(&self,", self.gui)
        self.assertIn("fn dragged(&self,", self.gui)

    def test_numeric_fields_keep_saved_keys(self):
        ranges = re.findall(r'\brange\("([^"]+)"', self.gui)
        self.assertGreater(len(ranges), 10)
        self.assertEqual(len(ranges), len(set(ranges)))
        self.assertIn("unlimited_max_level", ranges)
        self.assertIn("respawn_ticks", ranges)
        self.assertIn("clockWidthOffset", ranges)
        self.assertIn("affordable_cost_cap", ranges)


if __name__ == "__main__":
    unittest.main()
