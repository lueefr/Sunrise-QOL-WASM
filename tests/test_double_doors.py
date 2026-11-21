from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DoubleDoorsTests(unittest.TestCase):
    def test_manual_click_does_not_wait_for_server_tick(self):
        source = (ROOT / "src/doors.rs").read_text()
        click = source.split("pub fn clicked(&self,", 1)[1].split("pub fn changed(&self,", 1)[0]
        self.assertIn("self.process(host, task, true);", click)
        self.assertIn('event.click.hand != Hand::Main', click)
        self.assertIn('player.is_secondary_use_active(host) == Some(true)', click)
        self.assertIn('name == "minecraft:iron_door"', click)
        self.assertIn('self.bypass_click(&dim, pos);', click)
        self.assertNotIn("pending.push_back", click)
        self.assertNotIn("ServerTickPhase", click)

    def test_non_click_changes_still_use_bounded_fallback(self):
        source = (ROOT / "src/doors.rs").read_text()
        changed = source.split("pub fn changed(&self,", 1)[1].split("fn set_open(&self,", 1)[0]
        self.assertIn("pending.push_back(Change", changed)
        self.assertIn("if self.writing.load(Ordering::Acquire)", changed)
        self.assertIn("MAX_PENDING", source)
        self.assertIn("fn tick(&self,", source)
        self.assertIn("self.process(host, task, false);", source)

    def test_upstream_rules_are_explicit(self):
        source = (ROOT / "src/doors.rs").read_text()
        for option in ('"recursive_doors"', '"door_radius"', '"gates"', '"trapdoors"'):
            self.assertIn(option, source)
        self.assertIn('prop(&text, "half") == Some("upper")', source)
        self.assertIn('BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS', source)


if __name__ == "__main__":
    unittest.main()
