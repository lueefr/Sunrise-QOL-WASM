# Feature parity matrix — Sunrise QOL 0.4.0

There are 25 entries in the inherited suite catalog. 24 now have **source paths**,
including deliberately scoped or incomplete ones, and one has no implementation.
The separate Leaf Decay module is not counted among those 25. **Zero modules have
been certified as 1:1 in-game.** The source has not been compiled in this environment.

**A feature appearing in this matrix or the Suite Modules screen is NOT proof of runtime compatibility.**
Missing ABI exports in the bundled Sunrise `sunrise-wasm-std.wasm` will cause the
installer to stop before replacing the installed mod.

| Module | Source status | Missing / constraints |
|---|---|---|
| Automatic Doors | Initial | Proximity opening and delayed closure of doors this module opened; authoritative server only, in-game parity not yet verified |
| Beacon Placement | Initial | Right-click with supported vanilla mineral blocks fills empty pyramid positions and spends inventory; optional base recovery; no replacement of occupied blocks or parity for modded minerals |
| Conduit Placement | Initial | Right-click conduit with prismarine blocks builds a bounded 42-block frame in water or air; consumes held items; frame recovery configurable; no right-click-water placement or occupied-block replacement |
| Spawner Control | Initial | A read-only spawn position veto for normal spawner blocks surrounded by 5 torches (N/E/S/W/top). Does not reset the spawner's internal delay; other spawn sources unaffected |
| Bottled Air | Initial | Adds up to 150 configurable units of oxygen on vanilla glass bottle use while submerged; preserves vanilla item use. Water-potion restrictions and fire-offhand behaviors not implemented |
| Command Help | Initial | Captures the commands a server sends each player and paginates the allowed list when /help is used. Falls back to vanilla if no snapshot. Syntax suggestions, styled colors and full original options not yet supported |
| Inventory Totem | Initial | Vanilla totem only; death protection API must be tested |
| Inventory Mending | Initial | Repairs inventory items on XP-gain hook; differs from Mending orb selection |
| Double Doors | Initial | Exact-class block sync; redstone/AI timing is not 1:1 |
| Infinite Trading | Initial | Trade and demand updates; validate in multiplayer |
| Shulker Drops Two | Initial | Loot-table interception; validate loot interactions |
| Respawning Shulkers | Initial | Reschedule after confirmed death; retry after loaded chunks only |
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
| First Join Message | Initial | Fixed text only, no editable multilingual/broadcast templates |
| Welcome Message | Initial | Fixed text only, limited server controls |
| Vein Miner | Initial | Bounded connected search; tool costs and player block events not fully ported |
| Tree Harvester | Initial | Bounded tree removal, original replant/durability timing not 1:1 |
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
