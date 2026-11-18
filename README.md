# Sunrise QOL

**An experimental, modular quality-of-life suite for [Sunrise](https://github.com/sunrise-org/sunrise), written in Rust for its sandboxed WebAssembly mod API.**

[Releases](https://github.com/lueefr/Sunrise-QOL-WASM/releases) · [Actions](https://github.com/lueefr/Sunrise-QOL-WASM/actions) · [Versioning rules](docs/RELEASES.md) · [Port parity matrix](docs/PARITY.md)

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
| Enchantments and anvils | Unlimited Enchantments, Affordable Anvil, Tax Free Levels, Enchantment Level Indicator, Enchantment Level LangPatch | Partial implementations; see the detailed gaps below |
| HUD and controls | GUI Clock, GUI Compass, Vanilla Zoom | Initial implementations; not exact reproductions of all original options |
| Messages | First Join Message, Welcome Message | Fixed messages; customizable templates and full server controls are missing |
| Missing | Full Brightness Toggle | No equivalent gamma/lighting control exposed by the pinned public WASM API |

The detailed module-by-module audit is in [docs/PARITY.md](docs/PARITY.md) **in the source archive**; it will become browsable here after the source import. The catalog is preserved at `docs/original-catalog.json`. Individual Sounds / Sound Controller is intentionally excluded.

### Unlimited Enchantments: requested behavior vs. present code

The requested feature set is:

- Combine equal-level enchantments beyond their vanilla cap (for example, Sharpness V + Sharpness V → Sharpness VI), up to a configurable limit of **255**.
- Allow anvil operations beyond the vanilla **Too Expensive!** threshold, with a configurable maximum XP cost.
- Permit `/enchant` to use levels above the enchantment's vanilla cap, up to the configured maximum.
- Expose only **Max enchantment level** and **Max anvil cost**, plus the module's ON/OFF switch.

**Not complete yet:** the existing `src/unlimited.rs` only upgrades equal-level combinations when vanilla has already returned a valid anvil output. It cannot recover a result rejected before `anvil_update`, and it does not override `/enchant`'s own max-level validation. Its anvil cost cap only applies when it successfully upgrades an enchantment.

### Leaf Decay

Leaf Decay is independent from Tree Harvester. It offers accelerated decay, matching leaf and log types, configurable persistent-leaf behavior, optional wood support, a per-tick work cap, and optional effects. It is inspired by [Leaves Be Gone](https://github.com/Fuzss/leaves-be-gone) and [Leaves Us In Peace](https://gitlab.com/supersaiyansubtlety/leaves_us_in_peace), but currently uses bounded server-side work rather than a byte-for-byte port of their mixin and chunk-serialization behavior.

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

## Automated releases and changelog

The [release workflow](.github/workflows/release.yml) is configured to build and publish a `.smod`, release notes and `SHA256SUMS.txt` when tracked **mod source or build inputs** change on `main`, and only after its checks pass. README-only and documentation-only changes should not trigger a new release.

This project follows a **custom version scheme, not standard Semantic Versioning**:

| Change | Example | Commit prefix |
| --- | --- | --- |
| Initial source release | `1.0.0` | — |
| New feature | `1.1.1 → 2.0.0` | `feat:` |
| Bug fix | `1.0.0 → 1.1.1` | `fix:` |
| Maintenance change | `2.0.0 → 2.0.1` | Other prefixes |

The final component counts update releases within the current major version. Feature additions take precedence over bug fixes when a batch contains both. Commit messages must accurately describe changes; the workflow does not understand the semantic meaning of arbitrary diffs. See [docs/RELEASES.md](docs/RELEASES.md).

**No compiled release should be described as available until a successful GitHub Actions run and a corresponding GitHub Release exist.**

## Porting roadmap

1. Import the entire source archive to the default branch; keep version metadata and CI files in sync.
2. Compile the WASM mod against the pinned SDK and verify that all imports resolve against the actual Sunrise game runtime.
3. Complete Unlimited Enchantments, including rejected anvil output recovery and the authoritative `/enchant` path.
4. Close the missing or partial API integrations for Full Brightness Toggle, Tax Free Levels, Superflat detection, arbitrary damageable items, and global enchantment-level translations.
5. Verify leaf decay, tree mining, loot, totems, mending, trading, doors, and multiplayer parity against upstream behavior.
6. Test the custom configuration interface in the main menu, in-game, with input devices, and after restart.
7. Publish only validated binary artifacts, with a changelog identifying supported and incomplete functionality.

## Credits and upstream projects

Sunrise QOL is an independent experimental port/reimplementation. Its feature catalog originates from earlier Horizon QoL work and Minecraft mods, especially projects by [Serilum](https://github.com/Serilum). Additional reference projects include [Unlimited Enchantments](https://github.com/SayNooty/Unlimited-Enchantments), [Tax Free Levels](https://github.com/Fourmisain/TaxFreeLevels), [Enchantment Level Language Patch](https://github.com/teddyxlandlee/enchlevel-langpatch), [Leaves Be Gone](https://github.com/Fuzss/leaves-be-gone), and [Leaves Us In Peace](https://gitlab.com/supersaiyansubtlety/leaves_us_in_peace).

Original projects retain their own licensing and attribution requirements. Do not assume that an implementation, test, or API hook proves 1:1 compatibility.
