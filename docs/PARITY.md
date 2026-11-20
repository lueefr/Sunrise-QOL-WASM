# Feature parity matrix — Sunrise QOL 0.4.0

There are 25 entries in the inherited suite catalog. 24 now have **source paths**,
including deliberately scoped or incomplete ones, and one has no implementation.
The separate Leaf Decay module and the additional Sunrise modules are not counted among those 25. **Zero modules have
been certified as 1:1 in-game.** The source has not been compiled in this environment.

**A feature appearing in this matrix or the Suite Modules screen is NOT proof of runtime compatibility.** See [UPSTREAM-AUDIT.md](UPSTREAM-AUDIT.md) for direct comparisons against Serilum's 26.3.0 Java sources.
Missing ABI exports in the bundled Sunrise `sunrise-wasm-std.wasm` will cause the
installer to stop before replacing the installed mod.

| Module | Source status | Missing / constraints |
|---|---|---|
| Right Click Harvest | Partial | Right-click mature vanilla and modded age-bearing crops with native loot/Fortune and replant preserving other block properties; break above the base for cactus/sugar cane. Low 5-block cross, mid 3×3, high 13-block diamond only for a mature crop that permits radius harvesting. Original six dynamic tag datasets ship in the `.smod` without hardcoded overrides; item/block registry used for replant stats, world border checked, XP orbs spawned at player position even in creative, exact hoe component detection, configurable hunger/XP, one-point hoe wear, client planting sounds at player position and one-time hoe warning. Native Rust unit tests now run in both CI and release gates. **Not certified 1:1:** SDK lacks player RNG (uses native level RNG), non-destructive break/place event preflight, block clone-item mapping and cross-mod after-harvest callbacks. Modded crop subtype identification remains heuristic; live-world validation outstanding |
| Keep My Soil Tilled | Initial | Ported melon/pumpkin player-harvest hook and original 3×3 farmland check one layer below; converts #minecraft:dirt under the fruit back into farmland. Player-harvest timing, block permission and soil-state behavior require in-game validation |
| Softer Hay Bales | Initial | Registered an actual `BlockPart::fall_on` for hay with zero damage multiplier only for players. Toggle read on server tick, so newly changed settings can take up to one tick to propagate; not tested in gameplay |
| Nutritious Milk | Partial | Player-only finish event restores hunger and fractional saturation directly. Matching modded milk IDs supported; actual drink-path and multiplayer checks remain |
| Zombie-Proof Doors | Partial | Covers Zombie subclasses but not zombified piglins. Removes BreakDoor goal without changing normal door-opening navigation. A direct setCanBreakDoors flag, live toggles and goal reconstruction remain unresolved |
| Automatic Doors | Partial | Rewritten per-player-tick scan, integer-second timer, neighboring closure and sneaking checks from upstream 26.3.0; bounded player/tracked-door limits and gameplay timing still differ |
| Beacon Placement | Initial | Right-click with supported vanilla mineral blocks fills empty pyramid positions and spends inventory; optional base recovery; no replacement of occupied blocks or parity for modded minerals |
| Conduit Placement | Initial | Right-click conduit with prismarine blocks builds a bounded 42-block frame in water or air; consumes held items; frame recovery configurable; no right-click-water placement or occupied-block replacement |
| Spawner Control | Initial | A read-only spawn position veto for normal spawner blocks surrounded by 5 torches (N/E/S/W/top). Does not reset the spawner's internal delay; other spawn sources unaffected |
| Bottled Air | Partial | Restores air; blocks water-potion use when submerged; fire-item hand detects five items, consumes a water-potion from slots 35..1 and probabilistically gives back a glass bottle. Uses world RNG, rather than the source's Collective global RNG; real-game behavior needs validation |
| Painting Cycling | Partial | Replaces a painting entity with one using the next placeable same-size variant; supports backward cycling while crouching. Nested data-pack tags, FastPaintings blocks, and cycle particles are not implemented; validate entity recreation and UUID handling in-game |
| Passenger Removal | Partial | Original-success behavior on any nonempty passenger list, including when protected player passengers remain; now traverses all passengers without an arbitrary 8-person cap. Still needs actual engine and modded vehicle tests |
| Ground Elytra Fireworks | Experimental | Client lift on ground rocket use, then a vanilla glide action packet and a queued normal rocket use; only standard rockets and a chest-slot Elytra, no modded boosters. Depends on client/server movement timing and must be gameplay-tested |
| Extended Enchant Command | Partial | Intercepts only elevated /enchant @s levels 6–255 for a restricted set of vanilla gear and operators (permission >=2); queues the item mutation to the next server tick. All-target selectors, modded items and complete vanilla compatibility are not reproduced |
| Extended Bone Meal | Partial | Instant crouch growth for vanilla and registered modded age-based crops; nether wart grows one stage even crouched; tall crop/vine paths retained. Vanilla crop callback parity, dispensers and Kelp Fertilizer compatibility remain |
| Followers Teleport Too | Partial | Before /tp command, captures at most 32 nearby matching Owner UUID pets (not sitting or leashed); after a successful player teleport, attempts to move them to the target. No allays, mount remounting, temporary damage immunity or guaranteed interdimensional parity |
| Command Help | Partial | Per-player permitted root commands, configurable colors, clickable command suggestions and page navigation; falls back to vanilla for detailed syntax because the API does not expose Brigadier's usage tree or subcommand formatting |
| Drowned Prevention | Initial | Read-only drowned spawn veto near online players with Conduit Power in matching level; configurable horizontal radius 0–400. Gameplay validation pending |
| Hoe Tweaks | Partial | Tiers and crouching match 26.3.0 source; pumpkin/melon speed uses the public break-speed hook; arbitrary BushBlock detection, and click/placement side effects still need SDK/gameplay validation |\n| Inventory Totem | Partial | Honors the `minecraft:death_protection` item component, including modded/data-pack items with empty or supported clear/apply-effects payloads; increments the used-item stat and rolls back consumption on failed health write. Unsupported death effects, original vanilla advancement trigger, visual activation and bypass damage-tag matching remain unverified |
| Inventory Mending | Partial | One random eligible inventory item per XP event; guards equipped damaged Mending; exact orb timing, per-player RNG, and custom repair-with-XP effects are not exposed |
| Double Doors | Partial | Manual clicks synchronize paired blocks in the same server interaction (untested); redstone/AI still defer, and remote client prediction is missing. See [DOORS-SYNC.md](DOORS-SYNC.md) |
| Infinite Trading | Partial | Completed trade reset now also respects the original configurable minimum villager level (1–5) using Sunrise's merchant API. Still missing original pre-trade stock checks, multiplier, modded traders and recipe-aware trade-loop prevention |
| Shulker Drops Two | Partial | Every shell loot stack is now set to exactly the configured count and optional missing shell is added; late-spawned item entities are not intercepted |
| Respawning Shulkers | Partial | Captures and restores original entity NBT, health 30, new UUID and scheduled respawn; engine entity recreation, unloaded chunks and shutdown differ |
| Enchantment Level Indicator | Partial | Tooltip max-level suffix, with saved NBT and registered max-level lookup; modded tooltip layouts unverified |
| What Durability | Partial | damage_item parts for 67 common vanilla IDs plus tagged items; prevents future wear by default and preserves normal wear for IDs or #tags in itemsWithDurability; cannot rewrite already present damage or intercept arbitrary untagged third-party items |
| Always a Wither Skull | Initial | Drops hook only; alternate death routes unverified |
| Friendly Griefing | Initial | Entity lifecycle and griefing predicate hooks; incomplete mob-specific rules |
| GUI Clock | Initial | Minecraft clock time only; real computer clock and original advanced options not implemented |
| GUI Compass | Initial | HUD position and direction; exact display parity unverified |
| Vanilla Zoom | Initial | Native zoom hook with Controls binding; original Java item swap is not ported |
| Dragon Drops Elytra | Initial | Loot hook only, dragon death path requires validation |
| Hide Hands | Initial | Hand rendering hide rules; original item filter lists unavailable |
| Full Brightness Toggle | Not implemented | No public WASM API to set gamma or replace world lighting; adding screen tint would not reproduce it |
| Superflat World No Slimes | Partial (manual) | Blocks joining slimes ONLY when the user explicitly confirms flat-world status; no accessible world-generator flag |
| First Join Message | Partial | Configurable text with %s player-name replacement, 16 formatting colors and independent first-join broadcast; world-local first-join marker. Original spawn event timing and multiplayer broadcasts need real-game checks |
| Welcome Message | Partial | All three default messages, independent text/colors/HTTP URL click options, preceding blank line and dedicated-server-only switch from original; real lifecycle, UI input and integrated-server checks pending |
| Vein Miner | Initial | Bounded connected search; tool costs and player block events not fully ported |
| Tree Harvester | Partial | Bounded tree removal. Harvested logs trigger the shared Leaf Decay event/queue; fast decay requires the Leaf Decay module enabled and uses its own speed, per-tick cap and matching settings. Original replant/durability timing is not 1:1 |
| Ench. Level LangPatch | Partial | Formats missing enchantment/potion tooltip translation levels; does not modify global language storage |
| Tax Free Levels | Partial (anvil only) | One-level rename and existing-result limit; original flattened XP cost is NOT implemented |
| Unlimited Enchantments | Partial | Combines equal-level enchantments at anvil when vanilla made output, up to 255; not a registry or enchantment-table patch |
| Affordable Anvil | Initial | Modifies existing anvil output; cannot recreate result already rejected by vanilla |

## Outstanding 1:1 blockers

- **Full Brightness Toggle:** the pinned public Sunrise WASM client API exposes neither
  the gamma override nor an equivalent pre-lightmap shader hook. A translucent screen
  overlay is not physically equivalent and was intentionally NOT used.
- **Superflat World No Slimes:** the sandbox cannot read the server's world-generator
  preset. This implementation requires the explicit `Confirm This Is a Superflat World`
  setting, and blocks slime joins in the configured game instance. It does not infer
  world type from terrain patterns.
- **Tax Free Levels:** its original mixins change how XP costs are computed for anvil
  and enchanting payments, a source-specific path not exposed in the relevant public
  WASM events. Only cheap renaming and anvil preview cost cap are present.
- **What Durability:** registration-time ItemParts cover 67 named vanilla items and standard
  equipment tags. Registered items lose no future durability by default,
  whereas the `itemsWithDurability` allowlist (item IDs and #tags) preserves
  normal wear. Arbitrary untagged modded items and already-worn stacks need
  broader API interception.
- **Unlimited Enchantments:** internal component representation caps levels at 255;
  the module only upgrades matching enchantments on an anvil result the vanilla
  anvil has already produced. It cannot change the registry's maximum levels,
  ingredient eligibility, or results refused before the hook.
- **Enchantment Level LangPatch:** patches tooltip component keys but not resource
  language files globally. Other screens using `enchantment.level.N` are unaffected.

## Safety and configuration

All new modules start OFF; previously saved user preferences are retained. The
server-authoritative modules need the mod on the authoritative server, not only
on the client of a remote server. All effects use API hooks, never guessed
client-side writes to the server world. Features which require impossible
upstream internals are either explicitly labeled partial or absent from the menu.

No Sound Controller / Individual Sounds was re-added.

## Source provenance

- Serilum original sources: https://github.com/Serilum
- Leaves Be Gone: https://github.com/Fuzss/leaves-be-gone
- Leaves Us In Peace: https://gitlab.com/supersaiyansubtlety/leaves_us_in_peace
- Enchantment Level Language Patch: https://github.com/teddyxlandlee/enchlevel-langpatch
- Tax Free Levels: https://github.com/Fourmisain/TaxFreeLevels
- Unlimited Enchantments: https://github.com/SayNooty/Unlimited-Enchantments
- Sunrise pinned API: https://github.com/sunrise-org/sunrise/tree/2d8126336e579777eabf9b2fd8b3d476236a04f2
