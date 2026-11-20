from pathlib import Path
import json
import hashlib
import re
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from wasm_link_check import symbols


class ProjectTests(unittest.TestCase):
    def test_catalog_is_faithful(self):
        catalog = json.loads((ROOT / 'docs/original-catalog.json').read_text())
        mods = catalog['modules']
        self.assertEqual(len(mods), 25)
        self.assertEqual([m['order'] for m in mods], list(range(25)))
        self.assertEqual(sum(len(m['options']) for m in mods), 93)
        self.assertEqual(mods[20]['name'], 'Tree Harvester')
        self.assertTrue(any(o['key'] == 'enableFastLeafDecay' for o in mods[20]['options']))

    def test_manifest_ids_and_pin(self):
        cargo = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        manifest = tomllib.loads((ROOT / 'sunrise-mod.toml').read_text())
        self.assertEqual(manifest['id'], 'horizon_qol_suite')
        self.assertRegex(cargo['package']['metadata']['sunrise']['sdk'],
                         r'^[0-9a-f]{40}$')
        self.assertEqual(manifest['version'], cargo['package']['version'])

    def test_only_opener_is_native(self):
        settings = (ROOT / 'src/settings.rs').read_text()
        self.assertEqual(settings.count('SettingSpec::choice('), 1)
        self.assertNotIn('SettingSpec::toggle(', settings)
        self.assertNotIn('SettingSpec::range(', settings)
        self.assertIn('"open_ui", "qol.config.open"', settings)
        self.assertIn('host.config_files().read(&path)', settings)
        self.assertIn('host.config_files().write(&path, value.as_bytes())', settings)
        self.assertNotIn('host.setting(', settings)
        self.assertNotIn('host.set_setting(', settings)
        self.assertIn('pub fn save(', settings)

    def test_translations_are_valid(self):
        for language in ('en_us', 'pt_br'):
            data = json.loads((ROOT / f'assets/horizon_qol_suite/lang/{language}.json').read_text())
            self.assertIn('qol.module.vein_miner', data)
            self.assertIn('qol.module.hide_hands', data)

    def test_grief_uses_entity_lifecycle(self):
        grief = (ROOT / 'src/grief.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        self.assertNotIn('host.entity_kind(', grief)
        for name in ('entity_join_level', 'entity_leave_level', 'entity_mob_griefing', 'server_stopped'):
            self.assertIn('fn ' + name + '(', hooks)
        self.assertIn('ids.binary_search(&event.entity.id())', grief)

    def test_no_hashbrown_hashmap_hashset(self):
        text = '\n'.join(p.read_text() for p in (ROOT / 'src').glob('*.rs'))
        for banned in ('HashSet', 'HashMap', 'hashbrown'):
            self.assertNotRegex(text, r'\b' + banned + r'\b')

    def test_nested_gui_registration(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        settings = (ROOT / 'src/settings.rs').read_text()
        modules = re.findall(r'ModuleSpec \{ key: "([^"]+)", label:', gui)
        options = re.findall(r'(?:toggle|range|choice)\("([^"]+)", "', gui)
        self.assertEqual(len(modules), 44)
        self.assertEqual(len(options), 104)
        self.assertIn('let _ = save(host, key,', gui)
        self.assertIn('let _ = save(host, option.key,', gui)
        self.assertNotIn('host.set_setting(', gui)
        self.assertEqual(len(set(modules) | set(options)), 148)
        for hook in ('input_key', 'client_tick_pre', 'screen_render_post',
                     'screen_mouse_button_pressed_pre', 'screen_mouse_button_released_pre',
                     'screen_mouse_dragged_pre', 'screen_key_pressed_pre',
                     'screen_char_typed_pre'):
            self.assertIn(f'fn {hook}(', hooks)

    def test_scroll_theme_and_short_name(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        manifest = tomllib.loads((ROOT / 'sunrise-mod.toml').read_text())
        self.assertEqual(manifest['name'], 'Sunrise QOL')
        self.assertIn('fn screen_mouse_scrolled_pre(', hooks)
        self.assertIn('fn wheel(&self,', gui)
        self.assertIn('fn move_bar(&self,', gui)
        self.assertIn('draw_menu_background(', gui)
        self.assertIn('widgets.draw(', gui)
        self.assertIn('push_clip(', gui)
        self.assertIn('footer.draw(', gui)
        self.assertNotIn('MODULES_PER_PAGE', gui)
        self.assertNotIn('OPTIONS_PER_PAGE', gui)
        self.assertNotIn('Action::Page(', gui)
        self.assertNotIn('Action::Options(', gui)

    def test_safe_installer(self):
        script = (ROOT / 'instalar.fish').read_text()
        self.assertIn('rust_symbols_not_exported_by_std', script)
        self.assertIn('Installation CANCELLED', script)
        self.assertIn('cp "$pacote" "$destino"', script)
        self.assertLess(script.index('Installation CANCELLED'), script.index('cp "$pacote" "$destino"'))
        self.assertNotIn('string match -qx', script)

    def test_owner_only_config_overlay(self):
        settings = (ROOT / 'src/settings.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        for statement in (
            '"open_ui", "qol.config.open", ["qol.config.open.value"], 0',
            'SettingSpec::choice(',
        ):
            self.assertIn(statement, settings)
        self.assertIn('fn setting_changed(', hooks)
        self.assertIn('event.mod_id == ID && event.key == "open_ui"', gui)
        self.assertIn('screen_id == "options"', gui)
        self.assertIn('self.overlay.store(true, Ordering::Relaxed)', gui)
        self.assertIn('self.overlay.store(false, Ordering::Relaxed)', gui)
        self.assertIn('event.key == "key.keyboard.escape"', gui)
        self.assertIn('host.observed_screen_id()', gui)
        self.assertNotIn('Config(i)', gui)
        self.assertNotIn('mouse Config coordinates', gui)

    def test_overlay_hides_native_options_rendering(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        self.assertIn('fn screen_render_pre(', hooks)
        self.assertIn('ScreenRenderPreEvent', hooks)
        self.assertIn('self.gui.render_pre(host, event)', hooks)
        self.assertIn('fn render_pre(&self, host: &dyn ClientHost, event: &mut ScreenRenderPreEvent', gui)
        self.assertIn('host.mod_screen() == Some(SCREEN_ID)', gui)
        self.assertIn('fn screen_render_post(', hooks)
        self.assertIn('draw_menu_background(', gui)
        self.assertIn('let title = "Quality of Life Settings";', gui)

    def test_leaf_decay_is_independent_and_server_sided(self):
        src = (ROOT / 'src/decay.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        mining = (ROOT / 'src/mining.rs').read_text()
        self.assertIn('fn block_changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent)', hooks)
        self.assertIn('self.decay.changed(level, event);', hooks)
        self.assertIn('self.decay.tick(host, event);', hooks)
        self.assertIn('ModuleSpec { key: "leaf_decay", label: "Leaf Decay"', gui)
        self.assertIn('block_random_tick(host, task.pos)', src)
        self.assertIn('fn leaf_match(', src)
        self.assertIn('fn log_match(', src)
        self.assertIn('fn supported(', src)
        self.assertIn('if !on(host, "leaf_decay") || !flag(host, "accelerateLeafDecay", true)', src)
        self.assertIn('amount(host, "minimumDecayTicks", 5., 0., 1200.)', src)
        self.assertIn('amount(host, "maximumDecayTicks", 20., 0., 1200.)', src)
        self.assertIn('amount(host, "amountOfLeavesBrokenPerTick", 5., 1., 64.)', src)
        self.assertNotIn('treeLeavesPerTick', src)
        self.assertNotIn('enableFastLeafDecay', src)
        self.assertNotIn('treeLeavesPerTick', gui)
        self.assertNotIn('toggle("enableFastLeafDecay"', gui)
        self.assertIn('player.destroy_block(host, p, false)', mining)
        self.assertIn('MAX_TASKS', src)
        self.assertIn('"persistent"', src)
        self.assertIn('"distance"', src)
        self.assertNotIn('LeafTask', mining)
        self.assertNotIn('fn scan_leaves(', mining)

    def test_native_backgrounds_are_contextual(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('let in_world = host.local_player().is_some();', gui)
        self.assertIn('"gui/inworld_menu_list_background"', gui)
        self.assertIn('"gui/menu_list_background"', gui)
        self.assertIn('event.draw.tiled_sprite(sprite, body, 32., 0xFFFF_FFFF)', gui)
        self.assertIn('draw_menu_background([0., 0., size[0], size[1]], in_world)', gui)
        self.assertNotIn('draw_menu_background([0., 0., size[0], size[1]], host.playing())', gui)

    def test_migration_preserves_legacy_settings(self):
        import tempfile
        from migrate_settings import keys, migrate
        self.assertEqual(len(keys(ROOT)), 139)
        with tempfile.TemporaryDirectory() as temp:
            game = Path(temp)
            previous = game / 'sunrise-mods' / '.sunrise-mod-data' / 'horizon_qol_suite'
            previous.mkdir(parents=True)
            (previous / 'vein_miner').write_text('true')
            (previous / 'amountofleavesbrokenpertick').write_text('7')
            (previous / 'enablefastleafdecay').write_text('false')
            (previous / 'open_ui').write_text('0')
            (previous / 'junk').write_text('unsafe')
            self.assertEqual(migrate(game, ROOT), 3)
            target = game / 'config' / 'horizon_qol_suite'
            self.assertEqual((target / 'vein_miner').read_text(), 'true')
            self.assertEqual((target / 'amountofleavesbrokenpertick').read_text(), '7')
            self.assertFalse((target / 'junk').exists())
            self.assertEqual((target / 'accelerateleafdecay').read_text(), 'false')
            (target / 'vein_miner').write_text('false')
            self.assertEqual(migrate(game, ROOT), 0)
            self.assertEqual((target / 'vein_miner').read_text(), 'false')

    def test_new_modules_have_executable_hooks(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        doors = (ROOT / 'src/doors.rs').read_text()
        compass = (ROOT / 'src/compass.rs').read_text()
        self.assertIn('ModuleSpec { key: "double_doors", label: "Paired Doors", options: DOORS, category: Category::World }', gui)
        self.assertIn('ModuleSpec { key: "gui_compass", label: "Navigation Compass", options: COMPASS, category: Category::Hud }', gui)
        self.assertIn('fn right_click_block(', hooks)
        self.assertIn('self.doors.changed(level, event)', hooks)
        self.assertIn('self.doors.tick(host, event)', hooks)
        self.assertIn('self.automatic_doors.tick(host, event)', hooks)
        self.assertIn('self.beacon.broken(host, event)', hooks)
        self.assertIn('self.beacon.click(host, event)', hooks)
        self.assertIn('self.conduit.broken(host, event)', hooks)
        self.assertIn('self.conduit.click(host, event)', hooks)
        self.assertIn('spawner::check(host, event)', hooks)
        self.assertIn('bottled_air::use_bottle(HostSide::Server(host), event)', hooks)
        self.assertIn('self.help.sent(host, event)', hooks)
        self.assertIn('self.help.command(host, event)', hooks)
        self.assertIn('painting::interact(host, event)', hooks)
        self.assertIn('dismount::interact(host, event)', hooks)
        self.assertIn('self.elytra.clicked(HostSide::Client(host), event)', hooks)
        self.assertIn('self.elytra.tick(host, event)', hooks)
        self.assertIn('self.enchant_command.tick(host, event)', hooks)
        self.assertIn('extended_bonemeal::apply(side, event)', hooks)
        self.assertIn('self.followers.before(host, event)', hooks)
        self.assertIn('self.followers.after(host, event)', hooks)
        self.assertIn('conduit_drowned::check(host, event)', hooks)
        self.assertIn('fn gui_render_post(', hooks)
        self.assertIn('self.compass.render(host, event)', hooks)
        self.assertIn('level.set_block(host,', doors)
        self.assertIn('host.local_player()', compass)
        self.assertIn('event.batch.text(&content', compass)
        self.assertIn('MAX_CONNECTED', doors)
        self.assertIn('Hand::Main', doors)
        self.assertIn('Action::Cycle(option)', gui)

    def test_five_additional_modules_have_working_hook_paths(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        hooks = (ROOT / 'src/lib.rs').read_text()
        for name in ('inventory_totem', 'inventory_mending', 'respawning_shulkers',
                     'gui_clock', 'vanilla_zoom', 'enchantment_level_indicator',
                     'ench_level_langpatch'):
            self.assertIn('key: "' + name + '"', gui)
        for mod, call in [('totem', 'totem::on_death('), ('mending', 'mending::on_xp('),
                          ('shulkers', 'self.shulkers.tick('), ('clock', 'self.clock.render('),
                          ('zoom', 'zoom::current('), ('enchant', 'enchant::tooltip(')]:
            self.assertTrue((ROOT / f'src/{mod}.rs').exists())
            self.assertIn(call, hooks)
        self.assertIn('fn item_tooltip(', hooks)
        self.assertIn('TextContent::Translatable', (ROOT / 'src/enchant.rs').read_text())

    def test_remaining_features_are_not_fake(self):
        gui = (ROOT / 'src/gui.rs').read_text()
        src = (ROOT / 'src/lib.rs').read_text()
        matrix = (ROOT / 'docs/PARITY.md').read_text()
        self.assertEqual(gui.count('ModuleSpec { key:'), 43)
        self.assertNotIn('key: "full_brightness_toggle"', gui)
        self.assertIn('No public WASM API', matrix)
        self.assertIn('superflat::on_join(host, event)', src)
        self.assertIn('durability::register(r)', src)
        self.assertIn('enchant::tooltip(host, event.stack, event.lines)', src)
        self.assertIn('crate::unlimited::anvil(host, event)', (ROOT/'src/anvil.rs').read_text())
        self.assertIn('let active = on(host, module.key);', gui)
        self.assertIn('if !on(host, "leaf_decay")', (ROOT/'src/decay.rs').read_text())

    def test_outstanding_modules_documented(self):
        docs = (ROOT / 'docs/PARITY.md').read_text()
        for feature in ('Full Brightness Toggle', 'Superflat World No Slimes',
                        'Tax Free Levels', 'What Durability', 'Unlimited Enchantments'):
            self.assertIn(feature, docs)
        self.assertIn('NOT implemented', docs)
        self.assertIn('not been compiled', docs)

    def test_english_only_ui(self):
        source = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('let title = "Quality of Life Settings"', source)
        self.assertIn('Text::literal("Done")', source)
        self.assertIn('No Additional Options', source)
        self.assertNotRegex(source, r'Configura(?:ções|coes)|Conclu[ií]do|Folhas|[ÁáÉéÍíÓóÚúÇçÃãÕõÂâÊêÔô]')
        en = json.loads((ROOT / 'assets/horizon_qol_suite/lang/en_us.json').read_text())
        pt = json.loads((ROOT / 'assets/horizon_qol_suite/lang/pt_br.json').read_text())
        self.assertEqual(en, pt)
        self.assertEqual(en['qol.config.open'], 'Open settings')
        self.assertEqual(en['qol.config.open.value'], 'Open')
        self.assertEqual(tomllib.loads((ROOT / 'sunrise-mod.toml').read_text())['name'], 'Sunrise QOL')

    def test_wasm_parser(self):
        # One trivial core wasm module with a single exported function index 0.
        mod = bytes.fromhex('0061736d010000000104016000000302010007070103666f6f00000a040102000b')
        imports, exports = symbols(mod)
        self.assertEqual(imports, [])
        self.assertEqual(exports, [{'symbol': 'foo', 'kind': 0}])


if __name__ == '__main__':
    unittest.main()


class DurabilityRegressionTests(unittest.TestCase):
    def test_original_allowlist_semantics_and_tag_coverage(self):
        src = (ROOT / 'src/durability.rs').read_text()
        self.assertIn('if keeps_durability(host, item) { Flow::Continue } else { Flow::Cancel }', src)
        self.assertIn('fn is_allowed(', src)
        self.assertIn('host.tags().is_in_tag(item, key)', src)
        self.assertIn('r.item_part_tag(tag, Protect { item: None })', src)
        self.assertIn('fn allowlist_keeps_wear_for_ids_and_tags()', src)

    def test_anvil_cost_changes_only_after_upgrading(self):
        src = (ROOT / 'src/unlimited.rs').read_text()
        self.assertIn('if !changed { return; }', src)
        self.assertLess(src.index('if !changed { return; }'), src.index('let max_cost = amount('))
        self.assertIn('event.update.output = Some(stack);', src)

    def test_soil_and_hay_implement_real_game_hooks(self):
        hooks = (ROOT / 'src/lib.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        soil = (ROOT / 'src/soil.rs').read_text()
        hay = (ROOT / 'src/soft_hay.rs').read_text()
        self.assertIn('soil::harvested(host, event);', hooks)
        self.assertIn('soft_hay::register(r);', hooks)
        self.assertIn('soft_hay::refresh(host);', hooks)
        self.assertIn('fn fall_on(&self, _: &mut dyn LevelRandomHost', hay)
        self.assertIn('registrar.block_part(key, SoftHay)', hay)
        self.assertIn('event.damage_multiplier = 0.', hay)
        self.assertIn('level.set_block(host, below, farmland', soil)
        self.assertIn('minecraft:melon', soil)
        self.assertIn('minecraft:pumpkin', soil)
        self.assertIn('key: "keep_soil_tilled"', gui)
        self.assertIn('key: "softer_hay_bales"', gui)

    def test_nutritious_milk_is_activated_only_after_consumption(self):
        hooks = (ROOT / 'src/lib.rs').read_text()
        milk = (ROOT / 'src/milk.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('fn living_entity_use_item_finish(', hooks)
        self.assertIn('milk::finished(side, event)', hooks)
        self.assertIn('let HostSide::Server(host) = side else { return; };', milk)
        self.assertIn('event.stack.item().id(host)', milk)
        self.assertIn('player.set_food_level(host,', milk)
        self.assertIn('player.set_saturation(host,', milk)
        self.assertIn('key: "nutritious_milk"', gui)
        self.assertIn('hungerLevelIncrease', gui)
        self.assertIn('saturationLevelIncrease', gui)

    def test_zombie_door_protection_uses_actual_goal_api(self):
        hooks = (ROOT / 'src/lib.rs').read_text()
        zombie = (ROOT / 'src/zombie.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('zombie::joined(host, event);', hooks)
        self.assertIn('mob.remove_goal(host, "BreakDoor")', zombie)
        self.assertIn('mob.set_can_open_doors(host, false)', zombie)
        self.assertIn('key: "zombie_proof_doors"', gui)

    def test_exact_original_harvest_tags(self):
        # Git blob IDs from JamCoreModding/right-click-harvest (MIT).
        original_blobs = {
            'block/blacklist.json': '27b94f91030e05503db9f3d6da625e4d041bd3ea',
            'block/hoe_never_required.json': '0512ed8b0b2615998c2491ba9cb9bbb2601611f1',
            'block/radius_harvest_blacklist.json': '7f2a23f642ffe72a7d11d0255576091631ffdbb5',
            'item/low_tier_hoes.json': '848f378cee4d7c546408234710645f1e655d4bff',
            'item/mid_tier_hoes.json': '320dc441260333a7a416991fc94ab471f40e63d1',
            'item/high_tier_hoes.json': '76e37a23e87fe7eb9eff1ea35ba723a7eb2cb260',
        }
        for filename, expected in original_blobs.items():
            data = (ROOT / 'data/rightclickharvest/tags' / filename).read_bytes()
            sha = hashlib.sha1(b'blob ' + str(len(data)).encode() + bytes([0]) + data).hexdigest()
            self.assertEqual(sha, expected, f'{filename} must match upstream byte for byte')

    def test_upstream_harvest_tags_and_component_support(self):
        original = ROOT / 'data/rightclickharvest/tags'
        for kind in ('block', 'item'):
            self.assertTrue((original / kind).is_dir())
        for filename in ('blacklist', 'hoe_never_required', 'radius_harvest_blacklist'):
            data = json.loads((original / 'block' / f'{filename}.json').read_text())
            self.assertTrue(data['values'])
        for filename, expected in (
            ('low_tier_hoes', 'minecraft:iron_hoe'),
            ('mid_tier_hoes', 'minecraft:diamond_hoe'),
            ('high_tier_hoes', 'minecraft:netherite_hoe'),
        ):
            data = json.loads((original / 'item' / f'{filename}.json').read_text())
            names = [v['id'] if isinstance(v, dict) else v for v in data['values']]
            self.assertIn(expected, names)
        self.assertIn('Copyright 2026 Jamalam',
                      (ROOT / 'licenses/right-click-harvest.MIT').read_text())
        source = (ROOT / 'src/right_harvest.rs').read_text()
        self.assertIn('ComponentId::new("minecraft:block_transformer")', source)
        self.assertIn('Tag::String(value)', source)
        self.assertIn('level.with_random(host,', source)
        self.assertIn('wear_needed(kind, before_damage, current)', source)

    def test_right_click_harvest_hook_and_original_options(self):
        hooks = (ROOT / 'src/lib.rs').read_text()
        module = (ROOT / 'src/right_harvest.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('right_harvest::click(host, event)', hooks)
        self.assertIn('right_harvest::client_click(host, event)', hooks)
        self.assertIn('SoundPlay::at(sound, SoundSource::Players, origin)', module)
        self.assertIn('fn starts_radius(', module)
        self.assertIn('fn harvest_positions(', module)
        self.assertIn('level.world_border(host)', module)
        self.assertIn('player.entity().position(host)', module)
        self.assertIn('player.has_infinite_materials(host)', module)
        self.assertIn('let _ = pay_xp(host, player, cost)', module)
        self.assertIn('let _ = level.award_experience(host, x, y, z, points)', module)
        self.assertNotIn('if !creative {\n        match xp', module)
        self.assertNotIn('id == "minecraft:netherite_hoe"', module)
        self.assertNotIn('matches!(id.as_str(), "farmersdelight:tomatoes"', module)
        self.assertIn('starts_radius(crop, blocked, hoe, radius)', module)
        self.assertIn('minecraft:item.nether_wart.plant', module)
        self.assertIn('minecraft:item.crop.plant', module)
        self.assertIn('host.chat_line(Text::literal(', module)
        self.assertIn('rchHoeWarned', module)
        self.assertIn('key: "right_click_harvest"', gui)
        for key in ('rchRequireHoe', 'rchHarvestInRadius',
                    'rchHungerLevel', 'rchExperienceType'):
            self.assertIn(f'"{key}"', gui)
        self.assertIn('level.block_drops(host, pos, held, Some(player.entity()))', module)
        self.assertIn('subtract_replant(host, &id, state.block(), &mut drops)', module)
        self.assertIn('host.block_item(block)', module)
        self.assertIn('host.item_block(stack.item()) == Some(block)', module)
        self.assertIn('level.destroy_block(host, position, true, held, false)', module)
        self.assertIn('player.destroy_block(host, position, false)', module)
        self.assertNotIn('id.contains("seed")', module)
        self.assertIn('BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS', module)
        self.assertIn('InteractionResult::SuccessServer', module)
        self.assertIn('rightclickharvest:radius_harvest_blacklist', module)
        self.assertIn('player.is_secondary_use_active(host)', module)

    def test_trading_original_level_guard(self):
        src = (ROOT / 'src/trading.rs').read_text()
        gui = (ROOT / 'src/gui.rs').read_text()
        self.assertIn('host.merchant(entity)', src)
        self.assertIn('merchant.level(host)', src)
        self.assertIn('minimumVillagerLevel', src)
        self.assertIn('range("minimumVillagerLevel", "Minimum Villager Level", 1., 1., 5., 1.)', gui)

    def test_totem_reverts_item_if_health_write_fails(self):
        src = (ROOT / 'src/totem.rs').read_text()
        self.assertIn('if !living.set_health(host, 1.) {', src)
        self.assertIn('player.set_inventory_slot(host, slot, Some(original))', src)
        self.assertIn('ItemResource::of(&stack).value(host, &component)', src)
        self.assertIn('minecraft:death_protection', src)
        self.assertIn('DeathEffect::Clear', src)
        self.assertIn('DeathEffect::Apply', src)
        self.assertNotIn('stack.item().id(host) == "minecraft:totem_of_undying"', src)
