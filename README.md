# Sunrise QOL

**A modular quality-of-life suite for Sunrise, developed in Rust and WebAssembly.**

Sunrise QOL is primarily an **incomplete port and reimplementation of Serilum's quality-of-life bundle**, adapted to the Sunrise runtime. It also includes several independent convenience features. This is not an official Serilum release; feature and configuration parity remain in development.

> **Development status — not a stable release.** Successful Rust/WASM builds do not certify in-game behavior or feature parity. The release pipeline now also checks WASM imports and loads the actual .smod with Sunrise's runtime, but multiplayer correctness, GUI behavior and complete-world gameplay still require game-level tests. Do not assume a module works merely because it appears in the configuration interface.

## Automatic Sunrise SDK updates

GitHub Actions checks the official Sunrise `nightly` release on every build and every
six hours. It only selects a commit whose corresponding published `sdk-<sha>`
release exists with the Linux API and integrity metadata. The build tool comes
from the **same Sunrise revision**, rather than a separately pinned old commit.

If the nightly SDK changed, the pipeline temporarily repins `Cargo.toml`,
builds the mod against that SDK, checks guest imports and executes the real
Wasmtime sandbox. **Only after all checks pass** does the release job commit the
new SDK pin, advance the version and publish the matching `.smod`. A failed
update leaves the existing release and repository pin intact. An upstream
nightly with no published matching SDK is rejected rather than silently
producing a mod built against the old SDK. This validates ABI compatibility
with the official nightly, but cannot guarantee compatibility with a custom
or older local Sunrise build.

## Runtime sandbox

Before publishing a release, GitHub Actions builds the guest, checks its imports
against the pinned shared WASM standard, then runs it through the **real Sunrise
Wasmtime loader** at the exact SDK commit. The runtime test also executes in-memory command and entity-event scenarios:
command pagination, enchantment limits, villager griefing, superflat slime veto,
merchant offer resets, anvil XP-cost changes, concrete item-drop modifications for Wither Skeletons, Ender Dragons and Shulkers, client tooltip localization and enchantment cap indicators, first-person hand cancellation, zoom input/strength, welcome chat and first-join world-local markers. These are
real guest-side behavior checks, but **not** a complete Minecraft world,
multiplayer session or graphical test.

See [sandbox design and limitations](docs/SANDBOX.md) or run the
[standalone sandbox workflow](https://github.com/lueefr/Sunrise-QOL-WASM/actions/workflows/sandbox.yml).

## Features
**Checklist legend:** `[x]` means a matching implementation is present in the source archive, even if partial; `[ ]` means the behavior has not been ported. **No checked item has been certified as 1:1. Selected real-game observations are recorded in docs/validation/.** Some entries require new Sunrise hooks.

### Bundle-inspired quality-of-life features

- [x] Guaranteed skull drops when a wither skeleton dies *(initial support)*
- [x] Nearby doors open automatically and close after a delay *(initial support; in-game validation pending)*
- [x] Beacon bases can be placed by using mineral blocks on a beacon *(initial support; only empty positions, in-game validation pending)*
- [x] Conduits and their surrounding blocks become easier to place *(initial support; frame blocks placed only in water or empty spaces, in-game validation pending)*
- [x] Mob spawners can be disabled with a torch arrangement *(initial support; five surrounding torches, in-game validation pending)*
- [x] Glass bottles can temporarily restore underwater air *(oxygen restoration, water-potion restrictions and fire-offhand handling; gameplay validation pending)*
- [x] The help command gains clearer, paginated output *(initial support: configurable colors and clickable command/page links; detailed command syntax uses vanilla)*
- [x] Active conduit effects suppress nearby drowned spawning *(initial support: read-only spawn veto near Conduit Power players; in-game validation pending)*
- [x] Placed paintings can be cycled without breaking them *(initial support: same-size placeable variants from the server data pack; painting entity is replaced without drops; gameplay validation pending)*
- [x] Passengers can be removed from mounted entities more conveniently *(initial support: sneak + right-click, protects player passengers by default)*
- [x] Fireworks can launch Elytra flight directly from the ground *(experimental client sequence: upward lift, vanilla glide request and rocket use; runtime validation pending)*
- [x] Commands can apply enchantments above vanilla level limits *(partial: operators can run /enchant @s <enchantment> 6–255 on supported vanilla gear; standard lower levels and other selectors remain vanilla)*
- [x] Bone meal supports additional plants and faster growth *(one-stage nether wart, registered modded age-based crops, cactus, sugar cane and vines; dispenser and gameplay parity pending)*
- [x] Tamed companions follow their owners during command teleportation *(partial: capped scan of nearby owned, non-sitting, unleaded vanilla pets during /tp; no damage immunity, allay, or mounted-player parity yet)*
- [x] Metal equipment can be smelted back into materials *(26 native Furnace Recycle recipes; enable and /reload)*
- [x] Hoes can till and undo-till soil using the original six material-tier radii, crouching behavior, and durability costs *(Hoe Tweaks; crop break-speed replacement is now wired through the Sunrise break-speed event; some bush-block types and in-game parity remain unverified)*
- [x] Undying totems can activate from inventory slots *(partial)*
- [x] Farmland stays tilled after harvesting melons or pumpkins *(original 3×3 farmland check; gameplay validation pending)*
- [x] Kelp can be used as a fertilizer *(native growth helper on right click; consumes on successful use)*
- [x] Boats can be moved without breaking them *(sneak-right-click to pick up, stand then crouch to release; `/qolcarry boat` and `/qolcarry drop` provide a server alternative)*
- [x] Minecarts can be repositioned without breaking them *(same carrying controls; `/qolcarry minecart` provides a server alternative; native cargo support depends on the runtime)*
- [ ] Animals can be tempted again without a waiting period
- [x] Slow-falling effects and Feather Falling boots prevent player farmland trampling *(equipment/effect snapshot each tick; mobs pending)*
- [x] Drinking milk restores configurable hunger and saturation *(finished-consumption hook; gameplay validation pending)*
- [x] Connected ore veins can be harvested together *(partial)*
- [ ] Paths can be created across a selected route
- [ ] Some utility blocks can be accessed without placing them
- [ ] Fishing lines can be recast automatically after a catch
- [x] Mature crops can be harvested with right click and automatically replanted *(Right Click Harvest; basic vanilla crops, sugar cane, cactus and age-based modded crops on farmland; Fortune through original loot tables, configurable hoe requirement, tiered radius, hunger and XP; API/gameplay parity pending)*
- [x] Shulkers can return after a configurable respawn delay *(partial)*
- [ ] Connected scaffolding drops are collected near the broken block
- [x] Allowed sleeping times can be adjusted *(Sleep Sooner; configurable earliest time, default 12000)*
- [x] Landing on hay bales can eliminate fall damage *(registered hay landing hook for players; gameplay validation pending)*
- [x] Used-up hand stacks refill from inventory automatically *(right-click use, both hands; matching item components from main inventory)*
- [ ] Lapis lazuli can stay in an enchanting table for reuse
- [x] Slime spawning can be disabled in superflat worlds *(partial; requires manual world confirmation)*
- [x] Mushrooms can be planted under bright surface conditions *(solid surface right-click; vanilla later survival checks)*
- [x] Leggings protect players from sweet berry bush damage *(any nonempty leg slot; snapshot each tick)*
- [x] Entire trees can be felled with accelerated leaf decay and replanting *(partial)*
- [ ] Enchanting tables can function underwater with surrounding bookshelves
- [x] Village bells receive a crafting recipe *(enable and /reload)*
- [x] Wool, carpets and beds can be dyed in the world and through crafting; one wool can become four strings *(Wool Tweaks; reload-dependent recipe admission, custom subclasses and multiplayer parity remain partial)*
- [ ] Death drops can be protected inside a chest with equipment displayed
- [x] Zombies can be prevented from breaking doors *(removes BreakDoor goal when zombies join; existing-mob and multiplayer tests pending)*

### Additional quality-of-life features

- [x] Open or close paired doors, trapdoors, and fence gates together (initial support).
- [x] Allow repeat villager and wandering-trader trades by resetting offer uses (initial support).
- [x] Increase shulker shell drops to a configurable minimum (initial support).
- [x] Repair damaged inventory items using earned experience (partial).
- [x] Display an indicator when enchantments reach their normal maximum levels (partial).
- [x] Control future durability loss for supported items with configurable filters (partial).
- [x] Restrict selected mob griefing behaviors (partial).
- [x] Show an in-game clock on the HUD (initial support).
- [x] Show a direction compass on the HUD (initial support).
- [x] Zoom the camera using an assigned control (initial support).
- [x] Add an Elytra to dragon loot (initial support).
- [x] Hide the first-person hand rendering when configured (initial support).
- [x] Display first-join and repeat-join messages (initial support).
- [x] Format high enchantment and potion levels in item tooltips (partial).
- [x] Upgrade matching enchantments beyond their vanilla caps in valid anvil results, up to a configurable limit (partial).
- [x] Limit experience costs for supported anvil results (partial).
- [x] Accelerate leaf decay independently from tree harvesting, with configurable limits (partial).

### Additional features not yet available

- [ ] Extract enchantments from equipment onto books, or separate enchanted books using a grindstone.
- [ ] Toggle full-bright world lighting without changing the game's normal gamma limits.

Original upstream references for this batch: [Cycle Paintings](https://github.com/Serilum/Cycle-Paintings), [Dismount Entity](https://github.com/Serilum/Dismount-Entity) and [Easy Elytra Takeoff](https://github.com/Serilum/Easy-Elytra-Takeoff). The next three features reference [Extended Bone Meal](https://github.com/Serilum/Extended-Bone-Meal), [Followers Teleport Too](https://github.com/Serilum/Followers-Teleport-Too), and the existing Unlimited Enchantments implementation.

### Right Click Harvest

Based on [JamCoreModding/right-click-harvest](https://github.com/JamCoreModding/right-click-harvest) (MIT). The independent Sunrise WASM implementation supports mature age-based crops, cocoa/nether wart, cactus and sugar cane. When the player right-clicks a **mature, non-radius-blacklisted crop**, the configured hoe tiers harvest a 5-block cross (low), 3×3 square (mid) or 13-block diamond (high); the clicked block is handled last. Immature crops, cactus and sugar cane do not start an area harvest. Drops are rolled through Sunrise's real block loot API (including the held tool and its Fortune effects) and one replant item is removed from normal crop drops. The options **Require Hoe** (OFF), **Harvest in Radius** (ON), **Hunger Level** (None/Low/Normal/High) and **Experience Type** (None/Cost/Reward) default to the original values. The whole module is OFF by default.

**Progress on parity:** all six original block/item tag datasets are bundled in the `.smod` (see `data/rightclickharvest/tags/`), including tiered modded hoes and the upstream crop blacklist. Client-side planting sounds and the one-time hoe warning are implemented. Hoes are checked using the vanilla tag, bundled tier tags, and the block-transformer component. Modded age-bearing crops are no longer restricted to vanilla farmland. XP is now sampled with Sunrise's authoritative level RNG instead of an independent counter. The original source's license is preserved in `licenses/right-click-harvest.MIT`.

**Remaining blockers for exact 1:1:** Sunrise's public WASM API does not currently expose the player's own random stream, a cancellable preflight break/place check for a right-click crop without destroying it, the crop block's vanilla clone-item lookup, or a way to publish the original JamLib/Fabric post-harvest callback to other mods. Some modded blocks also lack a CropBlock subtype query. The packaged runtime test does not substitute for live gameplay validation.

## Settings and controls

The intended interface is entirely **English**, with one scrolling list of module sections, ON/OFF buttons, and options under each module. Modules default to **OFF** unless previously saved settings exist. Configuration values are stored privately under `config/horizon_qol_suite/`.

- In a world, press **F8**.
- Alternatively, navigate to **Options → Mods → Sunrise QOL → Config → Open settings → Open**.
- Use **Escape** or **Done** to close the custom view.

The native Config page contains only the opener; the suite settings appear in a custom overlay. This is **not** a true replacement of Sunrise's first Config click. The original Options screen remains logically active underneath the custom overlay, and controller navigation has not been verified.

## Building and installing from source

**Requirements:** a checked-out *complete source tree*, Arch Linux or another supported Linux installation, Fish shell for the included installer, Python 3, `unzip`, Rust 1.99.1, and the compatible Sunrise WASM build tool. The project pins the Sunrise API build to `2d8126336e579777eabf9b2fd8b3d476236a04f2`. Use a Sunrise game build with a matching WASM standard library.

The source archive includes an isolated Rust setup script for Fish:

```fish
fish preparar-rust.fish
fish instalar.fish
```

The installer compiles **the mod only**, checks the `.smod` structure and its Rust imports against the running Sunrise WASM standard library, migrates old settings without overwriting existing values, backs up any previously installed mod, and installs `horizon_qol_suite.smod` into the Sunrise game data directory. It refuses to replace an installed package if the link-compatibility check fails. Restart Sunrise after installation.

To inspect the package and run static regression tests:

```fish
fish diagnosticar.fish
python3 -m unittest discover -s tests -p 'test*.py' -v
```

**Passing Python tests does not establish successful WASM loading or gameplay parity.** An actual in-game test is still required.

## Credits and upstream projects

Sunrise QOL is an independent, incomplete port/reimplementation of Serilum's quality-of-life mod bundle for Sunrise, based on earlier Horizon QoL work. The original mod collection is by [Serilum](https://github.com/Serilum). Additional reference projects include [Unlimited Enchantments](https://github.com/SayNooty/Unlimited-Enchantments), [Tax Free Levels](https://github.com/Fourmisain/TaxFreeLevels), [Enchantment Level Language Patch](https://github.com/teddyxlandlee/enchlevel-langpatch), [Leaves Be Gone](https://github.com/Fuzss/leaves-be-gone), [Leaves Us In Peace](https://gitlab.com/supersaiyansubtlety/leaves_us_in_peace), and [Grind Enchantments](https://code.mschae23.de/mschae23/grind-enchantments). The automatic-door behavior references [Automatic Doors](https://github.com/Serilum/Automatic-Doors). Beacon placement references [Better Beacon Placement](https://github.com/Serilum/Better-Beacon-Placement). Conduit placement references [Better Conduit Placement](https://github.com/Serilum/Better-Conduit-Placement). Spawner control references [Better Spawner Control](https://github.com/Serilum/Better-Spawner-Control). Bottled air references [Bottled Air](https://github.com/Serilum/Bottled-Air). Command help references [Compact Help Command](https://github.com/Serilum/Compact-Help-Command). Drowned prevention references [Conduits Prevent Drowned](https://github.com/Serilum/Conduits-Prevent-Drowned).

Original projects retain their own licensing and attribution requirements. Do not assume that an implementation, test, or API hook proves 1:1 compatibility.


### In-game gameplay parity gate

A passing WASM build and synthetic-host smoke test **do not** imply 1:1 gameplay parity. The PR and release workflows now also run the built mod against the real Sunrise `Game` world and player-break path; both the resulting block states and positive/negative cases are checked. The first suite tests Keep My Soil Tilled. Coverage and untested modules are recorded in `gameplay-evidence.json`. The release is blocked if this real-engine gate fails; other modules are not represented as verified until they gain equivalent cases. See [in-game validation](docs/GAMEPLAY-VALIDATION.md).

The original First Join Message and Welcome Message options are now exposed in the suite UI: editable text and clickable HTTP(S) URLs, message colors, a per-world first-join broadcast, and dedicated-server gating. The Dismount Entity module also follows upstream success semantics without a passenger count cap. These are **source-level improvements**, not certified 1:1 gameplay parity.

### Wool Tweaks recipes

Enable **Wool Tweaks** in the QOL menu, then run `/reload` as the server operator to load its 49 recipes. Run `/reload` again after disabling it and after opening a world. Interaction dyeing follows the switch immediately. Recipe inputs, outputs and shapes match the original; recipe IDs use `horizon_qol_suite:wool_tweaks/`. This reload requirement differs from the original standalone mod. The same enable-then-`/reload` procedure applies to **Furnace Recycle** and **Village Bell Recipe**, also after entering a world.
