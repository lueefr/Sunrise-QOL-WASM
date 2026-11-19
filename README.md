# Sunrise QOL

**A modular quality-of-life suite for Sunrise, developed in Rust and WebAssembly.**

Sunrise QOL is primarily an **incomplete port and reimplementation of Serilum's quality-of-life bundle**, adapted to the Sunrise runtime. It also includes several independent convenience features. This is not an official Serilum release; feature and configuration parity remain in development.

> **Development status — not a stable release.** Version 1.0.0 is the initial version label, not a claim of feature parity. The source archive has passed structural Python checks, but successful Rust/WASM compilation, loading into Sunrise, multiplayer correctness, and gameplay parity are **not yet verified**. Do not assume a module works merely because it appears in the configuration interface.

## Features
**Checklist legend:** `[x]` means a matching implementation is present in the source archive, even if partial; `[ ]` means the behavior has not been ported. **No checked item has been certified as 1:1 or verified in-game.** Some entries require new Sunrise hooks.

### Bundle-inspired quality-of-life features

- [x] Guaranteed skull drops when a wither skeleton dies *(initial support)*
- [x] Nearby doors open automatically and close after a delay *(initial support; in-game validation pending)*
- [x] Beacon bases can be placed by using mineral blocks on a beacon *(initial support; only empty positions, in-game validation pending)*
- [x] Conduits and their surrounding blocks become easier to place *(initial support; frame blocks placed only in water or empty spaces, in-game validation pending)*
- [x] Mob spawners can be disabled with a torch arrangement *(initial support; five surrounding torches, in-game validation pending)*
- [x] Glass bottles can temporarily restore underwater air *(initial oxygen restoration on bottle use; original potion/fire-item rules pending, gameplay validation pending)*
- [x] The help command gains clearer, paginated output *(initial support: configurable colors and clickable command/page links; detailed command syntax uses vanilla)*
- [x] Active conduit effects suppress nearby drowned spawning *(initial support: read-only spawn veto near Conduit Power players; in-game validation pending)*
- [ ] Placed paintings can be cycled without breaking them
- [ ] Passengers can be removed from mounted entities more conveniently
- [ ] Fireworks can launch Elytra flight directly from the ground
- [ ] Commands can apply enchantments above vanilla level limits
- [ ] Bone meal supports additional plants and faster growth
- [ ] Tamed companions follow their owners during command teleportation
- [ ] Metal equipment can be smelted back into materials
- [ ] Hoes gain faster farming actions and soil-tilling controls
- [x] Undying totems can activate from inventory slots *(partial)*
- [ ] Farmland stays tilled after harvesting melon or pumpkin stems
- [ ] Kelp can be used as a fertilizer
- [ ] Boats can be moved without breaking them
- [ ] Minecarts can be repositioned without breaking them
- [ ] Animals can be tempted again without a waiting period
- [ ] Slow-falling effects prevent accidental farmland trampling
- [ ] Drinking milk restores configurable hunger and saturation
- [x] Connected ore veins can be harvested together *(partial)*
- [ ] Paths can be created across a selected route
- [ ] Some utility blocks can be accessed without placing them
- [ ] Fishing lines can be recast automatically after a catch
- [ ] Harvested crops can be replanted using their drops
- [x] Shulkers can return after a configurable respawn delay *(partial)*
- [ ] Connected scaffolding drops are collected near the broken block
- [ ] Allowed sleeping times can be adjusted
- [ ] Landing on hay bales can eliminate fall damage
- [ ] Used-up hand stacks refill from inventory automatically
- [ ] Lapis lazuli can stay in an enchanting table for reuse
- [x] Slime spawning can be disabled in superflat worlds *(partial; requires manual world confirmation)*
- [ ] Mushrooms can be planted under bright surface conditions
- [ ] Leggings protect players from sweet berry bush damage
- [x] Entire trees can be felled with accelerated leaf decay and replanting *(partial)*
- [ ] Enchanting tables can function underwater with surrounding bookshelves
- [ ] Village bells receive a crafting recipe
- [ ] Wool crafting, dyeing, and string conversion become more flexible
- [ ] Death drops can be protected inside a chest with equipment displayed
- [ ] Zombies can be prevented from breaking doors

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
