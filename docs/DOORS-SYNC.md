# Double Doors — comparison with upstream 26.3.0

Reference: [Serilum/Double-Doors at 26.3.0](https://github.com/Serilum/Double-Doors/tree/26.3.0), particularly `DoorEvent.java`, `Util.java`, `BlockStateBaseMixin.java`, and `ConfigHandler.java`. The upstream repository is marked **All Rights Reserved**. This code is an independent implementation of the behavior; no upstream source has been copied.

## Why there was a delay

Before this change, the Sunrise `block_changed` callback enqueued the source block and `server_tick` processed up to four queued changes. The second door therefore always waited until a later server phase, even for a normal right click.

The upstream mod hooks the door use *after* the block's open state changes and immediately propagates the new state to matching neighbors, in that interaction. Sunrise's public `block_changed` callback is **read-only** and cannot do the same world write there.

This change synchronizes matching neighbors in the authoritative server's `right_click_block` callback **before** the normal door use, using the inverse of the source's old `open` property. Vanilla then toggles the clicked block. The same action marks its source to suppress duplicate deferred work. Sneaking bypasses matching, as upstream does. Iron doors are excluded from manual toggling.

## Behavior status

| Interaction | Behavior after change | 1:1 status |
| --- | --- | --- |
| Main-hand wooden door click | Matching doors written synchronously in the click callback; clicked door left to vanilla | Needs gameplay validation |
| Sneak-click | Only the clicked door is left to vanilla | Needs gameplay validation |
| Trapdoors / fence gates | Same-callback neighbor writes when the corresponding option is enabled | Needs gameplay validation |
| Recursive connectivity, radius | Uses existing bounded neighborhood search, matching block identifiers | Approximation; original compares display names |
| Button / pressure plate / lever | Still uses the `block_changed` callback and a later tick for synchronization | **Not 1:1** |
| Villager/AI door interactions | Still uses later `block_changed` callback and a later tick | **Not 1:1** |
| Remote multiplayer rendering | Server writes happen in one action, but companion client-side prediction is not implemented | **Not 1:1** |
| Blacklisted modded doors, other-mod compatibility detection | Not exposed as the original configurable behavior | **Not 1:1** |

## Remaining platform requirement

Full parity needs a post-use door callback that can mutate the level in the same interaction, plus a client-side prediction/update path when the player uses a remotely hosted server. The pinned public Sunrise WASM API does not provide both of these as an exact substitute for the upstream NeoForge/Fabric mixins. Do not describe these cases as fully ported.

These checks verify source wiring only; they do not certify compilation against the pinned SDK or in-game behavior. The installer must continue to reject packages with incompatible WASM standard-library exports.
