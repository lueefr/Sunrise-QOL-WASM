# Sunrise QOL

**An experimental, modular quality-of-life suite for [Sunrise](https://github.com/sunrise-org/sunrise), written in Rust for its sandboxed WebAssembly mod API.**

Sunrise QOL is essentially an **incomplete port/reimplementation of [Serilum's QoL Bundle](https://modrinth.com/mod/QC7wpUjM)**, adapted for the Sunrise runtime. It is a separate project, not an official Serilum release or a repack of the original Java mods. Features and configuration parity remain work in progress.

[Releases](https://github.com/lueefr/Sunrise-QOL-WASM/releases) · [Source code](https://github.com/lueefr/Sunrise-QOL-WASM) · [Implementation status](docs/PARITY.md)

> **Development status — not a stable release.** Version 1.0.0 is the initial version label, not a claim of feature parity. The source archive has passed structural Python checks, but successful Rust/WASM compilation, loading into Sunrise, multiplayer correctness, and gameplay parity are **not yet verified**. Do not assume a module works merely because it appears in the configuration interface.

## Repository status

The repository was initialized with version metadata and a release workflow. **The original 1.0.0 source archive must also be imported** before the project can be compiled on GitHub. If the `src/` directory and the installation scripts are absent from the default branch, the import is incomplete, and the release workflow cannot produce a usable mod.

The mod ID is `horizon_qol_suite` for compatibility with older saves and settings. The visible mod name is **Sunrise QOL**.

## Features and implementation coverage

The source archive contains implementations or initial hook handlers for 24 of the 25 modules in the inherited suite catalog. **Leaf Decay is a separate additional module.** None of these implementations is certified as an exact 1:1 port.

| Area | Modules | Current limitations |
| --- | --- | --- |
| Trees and mining | Tree Harvester, Vein Miner, Leaf Decay | Bounded scanning and scheduled work; some original break-event, durability, replanting, chunk-persistence and tick semantics differ |
| Items and inventory | Inventory Totem, Inventory Mending, What Durability, Hide Hands | Initial or partial hook coverage; arbitrary third-party items, death handling and exact XP behavior require testing |
| Blocks and entities | Double Doors, Friendly Griefing, Superflat World No Slimes | Some AI/redstone cases differ; Superflat requires manual confirmation |
| Trading and drops | Infinite Trading, Shulker Drops Two, Respawning Shulkers, Always a Wither Skull, Dragon Drops Elytra | Validate loot tables, lifecycles, respawns and authoritative multiplayer behavior |
| Enchantments and anvils | Unlimited Enchantments, Affordable Anvil, Tax Free Levels, Enchantment Level Indicator, Enchantment Level LangPatch | Partial implementation; full gameplay parity has not been verified |
| HUD and controls | GUI Clock, GUI Compass, Vanilla Zoom | Initial implementations; not exact reproductions of all original options |
| Messages | First Join Message, Welcome Message | Fixed messages; customizable templates and full server controls are missing |
| Missing | Full Brightness Toggle; [Grind Enchantments](https://code.mschae23.de/mschae23/grind-enchantments) | Brightness control lacks a suitable public WASM hook. Transferring enchantments from equipment to books and separating enchanted books with a grindstone are not ported. |

The detailed module-by-module audit is in [docs/PARITY.md](docs/PARITY.md) **in the source archive**; it will become browsable here after the source import. The catalog is preserved at `docs/original-catalog.json`. Individual Sounds / Sound Controller is intentionally excluded.

### Reference feature catalog

The following **44 gameplay behaviors** are taken from the scope of [Serilum's official QoL Bundle](https://www.curseforge.com/minecraft/mc-mods/serilums-qol-bundle). They describe the **target of the incomplete port**, not 44 features already implemented or tested. Individual upstream mod names are deliberately omitted.

- Guaranteed skull drops when a wither skeleton dies.
- Nearby doors open automatically and close after a delay.
- Beacon bases can be placed by using mineral blocks on a beacon.
- Conduits and their surrounding blocks become easier to place.
- Mob spawners can be disabled with a torch arrangement.
- Glass bottles can temporarily restore underwater air.
- The help command gains clearer, paginated output.
- Active conduit effects suppress nearby drowned spawning.
- Placed paintings can be cycled without breaking them.
- Passengers can be removed from mounted entities more conveniently.
- Fireworks can launch Elytra flight directly from the ground.
- Commands can apply enchantments above vanilla level limits.
- Bone meal supports additional plants and faster growth.
- Tamed companions follow their owners during command teleportation.
- Metal equipment can be smelted back into materials.
- Hoes gain faster farming actions and soil-tilling controls.
- Undying totems can activate from inventory slots.
- Farmland stays tilled after harvesting melon or pumpkin stems.
- Kelp can be used as a fertilizer.
- Boats can be moved without breaking them.
- Minecarts can be repositioned without breaking them.
- Animals can be tempted again without a waiting period.
- Slow-falling effects prevent accidental farmland trampling.
- Drinking milk restores configurable hunger and saturation.
- Connected ore veins can be harvested together.
- Paths can be created across a selected route.
- Some utility blocks can be accessed without placing them.
- Fishing lines can be recast automatically after a catch.
- Harvested crops can be replanted using their drops.
- Shulkers can return after a configurable respawn delay.
- Connected scaffolding drops are collected near the broken block.
- Allowed sleeping times can be adjusted.
- Landing on hay bales can eliminate fall damage.
- Used-up hand stacks refill from inventory automatically.
- Lapis lazuli can stay in an enchanting table for reuse.
- Slime spawning can be disabled in superflat worlds.
- Mushrooms can be planted under bright surface conditions.
- Leggings protect players from sweet berry bush damage.
- Entire trees can be felled with accelerated leaf decay and replanting.
- Enchanting tables can function underwater with surrounding bookshelves.
- Village bells receive a crafting recipe.
- Wool crafting, dyeing, and string conversion become more flexible.
- Death drops can be protected inside a chest with equipment displayed.
- Zombies can be prevented from breaking doors.

Features can require different Sunrise hooks and may be missing, partial, or behaviorally different until separately validated. Individual sound-control functionality is intentionally outside this project's scope.

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

Sunrise QOL is an independent, incomplete port/reimplementation of Serilum's quality-of-life mod bundle for Sunrise, based on earlier Horizon QoL work. The original mod collection is by [Serilum](https://github.com/Serilum). Additional reference projects include [Unlimited Enchantments](https://github.com/SayNooty/Unlimited-Enchantments), [Tax Free Levels](https://github.com/Fourmisain/TaxFreeLevels), [Enchantment Level Language Patch](https://github.com/teddyxlandlee/enchlevel-langpatch), [Leaves Be Gone](https://github.com/Fuzss/leaves-be-gone), [Leaves Us In Peace](https://gitlab.com/supersaiyansubtlety/leaves_us_in_peace), and [Grind Enchantments](https://code.mschae23.de/mschae23/grind-enchantments).

Original projects retain their own licensing and attribution requirements. Do not assume that an implementation, test, or API hook proves 1:1 compatibility.
