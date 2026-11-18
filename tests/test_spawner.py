"""Spawner torches should stop only a mob from a normal spawner block."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class SpawnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/spawner.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_server_spawn_gate(self):
        self.assertIn("MobSpawnPositionCheckEvent", self.src)
        self.assertIn("SpawnReason::Spawner", self.src)
        self.assertIn("Spawner::Block { pos }", self.src)
        self.assertIn("SpawnCheck::Fail", self.src)
        self.assertIn('if !on(host, "spawner_control")', self.src)
        self.assertIn("spawner::check(host, event)", self.hooks)
        self.assertIn("event.level.block(host, location)", self.src)

    def test_five_torches_and_exact_guard(self):
        self.assertIn("fn neighbors(pos: [i32; 3]) -> Option<[[i32; 3]; 5]>", self.src)
        self.assertIn("z.checked_sub(1)", self.src)
        self.assertIn("x.checked_add(1)", self.src)
        self.assertIn("z.checked_add(1)", self.src)
        self.assertIn("x.checked_sub(1)", self.src)
        self.assertIn("y.checked_add(1)", self.src)
        self.assertIn("torch(&block.id(host))", self.src)
        self.assertIn('ModuleSpec { key: "spawner_control", label: "Spawner Control", options: &[]', self.gui)

    def test_original_list_order(self):
        readme=(ROOT/"README.md").read_text()
        self.assertLess(readme.index("Conduits and their surrounding blocks"),readme.index("Mob spawners can be disabled"))
        self.assertIn("- [x] Mob spawners can be disabled",readme)
        self.assertIn("- [x] Glass bottles can temporarily",readme)


if __name__=="__main__":
    unittest.main()
