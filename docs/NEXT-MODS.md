# Next original mods

The next batch starts after PR #8 merged at `c91a584ce82903f20c2cd50e4085f3ac21b37eb8`. None of these modules is certified 1:1.

## Wool Tweaks

Reference: [26.3.0 WoolClickEvent](https://github.com/Serilum/Wool-Tweaks/blob/26.3.0/Common/src/main/java/com/serilum/wooltweaks/events/WoolClickEvent.java).

Initial implementation: wool-tag blocks and the sixteen vanilla colored carpets, vanilla dyes, both hands, one dye consumed outside creative, same-color no-op, swing and continued vanilla interaction. The module starts OFF. Inventory-write refusal attempts to restore the original block.

Next: beds (both halves, facing and occupied state), crafting recipes, wool-to-string conversion, runtime custom carpet/bed class classification, and client prediction. Permission checks can differ from the source's unconditional callback. Actual block writes do not participate in the inventory transaction API; failure rollback is best effort and cannot undo external callbacks.

## Thorny Bush Protection

References: [SweetBerryBushMixin](https://github.com/Serilum/Thorny-Bush-Protection/blob/26.3.0/Common/src/main/java/com/serilum/thornybushprotection/mixin/SweetBerryBushMixin.java), [Util](https://github.com/Serilum/Thorny-Bush-Protection/blob/26.3.0/Common/src/main/java/com/serilum/thornybushprotection/util/Util.java), [ConfigHandler](https://github.com/Serilum/Thorny-Bush-Protection/blob/26.3.0/Common/src/main/java/com/serilum/thornybushprotection/config/ConfigHandler.java), branch 26.3.0.

Required behavior: interrupt only the berry bush's own hurt call for players, with `negateBushDamageForPlayers=true`; by default any nonempty LEGS stack qualifies, not a suffix check for a leggings item. Preserve slowing and all other damage.

SDK integration pending: a block's `entity_inside` Part cannot suppress its vanilla hurt call, and hit hosts do not lend the victim's live equipment as ServerHost does. Cancelling every `sweet_berry_bush` damage source would also affect commands and other mods, unlike the original. Equipment cached from the preceding tick can be stale and is not an exact substitute. No misleading working toggle is exposed.

## No Feather Trample

Reference: [26.3.0 FarmlandBlockMixin](https://github.com/Serilum/No-Feather-Trample/blob/26.3.0/Common/src/main/java/com/serilum/nofeathertrample/mixin/FarmlandBlockMixin.java).

Required behavior: before conversion to the base block, cancel for any LivingEntity with an effective Feather Falling level greater than zero or Slow Falling. Includes mobs, not just players. Ordinary dehydration with no entity remains unchanged.

SDK integration pending: `farmland_trample` exposes entity ID, block, position and distance with `LoopContext`; it does not provide a live living/effect/equipment view. An exact port also needs every corresponding base-block conversion path, not a delayed repair after farmland and its crop have already been destroyed.

## Kelp Fertilizer

References: [KelpEvent](https://github.com/Serilum/Kelp-Fertilizer/blob/26.3.0/Common/src/main/java/com/serilum/kelpfertilizer/events/KelpEvent.java), [BehaviourKelpDispenser](https://github.com/Serilum/Kelp-Fertilizer/blob/26.3.0/Common/src/main/java/com/serilum/kelpfertilizer/dispenser/BehaviourKelpDispenser.java), and Collective's [CropFunctions](https://github.com/Serilum/Collective/blob/26.3.0/Common/src/main/java/com/natamus/collective/functions/CropFunctions.java), branch 26.3.0.

Required behavior: the player path uses INTERACTION bonemeal semantics, consumes one kelp for any valid target even when its random growth attempt fails (except creative), then emits the level event and swing. The dispenser path uses growCrop, not growWaterPlant.

SDK's `level.bone_meal` is explicitly dispenser-style, includes growWaterPlant and returns whether growth succeeded. It cannot reproduce valid-target consumption or the original interaction source. Do not equate this helper with a complete port.
