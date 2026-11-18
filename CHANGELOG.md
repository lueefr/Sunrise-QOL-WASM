# 1.0.0 — Initial release

- Initial Sunrise QOL WASM source, re-versioned from the supplied 0.4.0 project.
- No claim of completed feature parity or successful in-game validation.
- Automatic releases require a successful WASM build.

# 0.4.0 — durability parity and hook correctness (experimental)

- Corrected What Durability's allowlist semantics to match Serilum's original: by default, registered damageable items no longer lose future durability; listed IDs and `#item_tags` retain vanilla wear.
- Added `item_part_tag` registrations for standard tool and enchantable tags, increasing coverage for modded items in those tags; the public ItemPart API still cannot intercept arbitrary unregistered items, reset damage already present or alter item tags on the server like the original mixin.
- Added Rust tests for allowlist IDs, absent entries, invalid entries and tagged entries.
- Fixed Unlimited Enchantments changing anvil costs even when no enchantment upgrade happened; the cost is now altered only after the stronger result is successfully created.
- Made Inventory Totem restore its original item on a failed health write.
- All suite module toggles remain OFF by default, old config stays in place.
- **No compiled `.smod` or verified in-game parity**: the matching pinned SDK and local Rust toolchain are required.

# 0.3.9 — expanded ports and honest status

- Hook handlers exposed through Suite Modules: Inventory Totem, Inventory Mending,
  Respawning Shulkers, GUI Clock, Vanilla Zoom.
- New source modules: Enchantment Level Indicator, Enchantment Level LangPatch,
  What Durability (67 common vanilla item IDs), Unlimited Enchantments (anvil),
  Superflat World No Slimes (manual confirmation).
- Tax Free Levels partial anvil rules (NOT the XP-curve algorithm).
- UI currently represents 24 of the 25 original modules, plus Leaf Decay.
- Fixed Leaf Decay's inconsistent default: all modules OFF when no saved value.
- Settings migration now covers 96 keys (including new module options).
- Full Brightness Toggle intentionally omitted: no public gamma/lighting hook.
- No claims of complete 1:1 parity; compile/run still require a matching Sunrise.

## 0.3.7 — Double Doors and GUI Compass (incremental ports)

- Ported a bounded, server-side Double Doors controller using `block_changed`, `right_click_block`, and `server_tick`: automatic syncing for pairs, configurable recursive connectivity, search distance, trapdoors, gates and sneak bypass.
- Ported GUI Compass to `gui_render_post`: actual client-player coordinates and facing, inventory compass gating, FXYZ/F/XYZ modes, positioning, text shadow and RGB channel controls.
- Added `Choice` widgets to the single-page custom menu; the native Config page still only provides its opener.
- Migrated the new setting keys through the existing safe, non-overwriting preference migration; kept the original 25-module catalog intact.
- Bounded Double Doors search to 256 connected blocks and four queued source transitions per tick, and cached GUI Compass config for 500 ms.
- Added static regression tests for both modules. **No WASM build or gameplay parity verified in this environment.**

## 0.3.6 — Leaf Decay

- Added independent Leaf Decay module, combining Leaves Be Gone's accelerated
  decay scheduling with Leaves Us In Peace-inspired species matching.
- Leaf tasks are triggered by server-side log and leaf removals, including
  automatic Tree Harvester cuts, using capped work queues and vanilla loot.
- Added configurable matching of leaf types, log types, persistent leaf modes,
  wood support, decay timing, decay effects and per-tick processing budget.
- Preserved original module menu, English-only UI and in-world transparency.
- Migrates prior fast-decay setting and leaf-processing budget without replacing
  existing settings. No changes to the game's native Config registration.
- See README for parity limitations versus the upstream Java mixins.

# Sunrise QOL 0.3.5

- Keep only **Open settings: Open** on the native Config page. All module switches and nested options are exclusive to **Suite Modules**.
- Store suite values in the mod-owned `config/horizon_qol_suite/` directory through `Host::config_files`, because Sunrise does not permit hidden `SettingSpec` entries and unregistered setting keys cannot be written with `Host::set_setting`.
- Migrate 35 legacy keys from `sunrise-mods/.sunrise-mod-data/horizon_qol_suite/` during Fish installation, without overwriting existing private settings or removing old files.
- Draw the native `gui/menu_list_background` or `gui/inworld_menu_list_background` behind the middle list, including a semi-transparent fallback.
- Detect an in-world UI via `ClientHost::local_player()` rather than `playing()` (which is false while screens are open), restoring the translucent in-world background.
- Suppress the host's own mod-screen painting as well as the Options overlay before drawing **Suite Modules**.
- All UI text remains in English; gameplay feature hooks were not changed.
- Passed 15 static Python tests. The `.smod` still requires a local Rust/SDK build and in-game verification.

# 0.3.4

- Added the public `screen_render_pre` hook to suppress Options drawing whenever Suite Modules is active, so the suite no longer visually stacks on the native Config page.
- Kept `screen_render_post` drawing the suite with the active resource pack's menu background and widgets.
- Centered **Suite Modules**, added native-style header/footer separators, and preserved the single scrollable list.
- Preserved owner-scoped `setting_changed`, the native Config entry, Escape / Done handling, and F8.
- Known limitation: this is still a visual overlay. The underlying Options page remains active in the game's state; gamepad interaction and the pre-Game startup menu are not fixed.
- Not compiled or tested in-game in this environment. No gameplay module changes.

# Sunrise QOL 0.3.3

- All user-visible UI text is in English (including the built-in Config entry, overlay, module labels, and option names).
- Both `en_us.json` and `pt_br.json` now provide the same English labels to avoid Portuguese entries when the game language is set to Portuguese.
- Updated the mod description and terminal diagnostics to English.
- Preserved the internal mod ID, setting keys, existing gameplay code, and Config overlay behavior.
- No claim of successful Rust compilation or in-game validation.

# Sunrise QOL 0.3.2

- Added a native **Open settings** entry using a single-option `Choice` and the owner-only `setting_changed` hook.
- Added an overlay to the Options screen using native menu textures and widgets without intercepting other mods.
- Added mouse, keyboard, and scroll handling for the overlay and closing with Escape or Done.
- Kept F8 as a world-only shortcut.

# Sunrise QOL 0.3.1

- Renamed the displayed title to Sunrise QOL while retaining the internal ID.
- Redesigned settings as a single vertically scrolling list, without pagination.
- Added mouse-wheel and draggable scrollbar support.
- Switched to Sunrise's native menu background, allowing active resource-pack textures.
- Separated the Done button from scrolling content and explicitly drew module headings.
- Documented the API limitation preventing a direct redirection of the initial Config click.
- No claims of fixed gameplay modules or full feature parity.
