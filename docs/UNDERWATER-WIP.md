# Unverified Underwater Enchanting adaptation

This branch is intentionally separate from the tested module batch. It compiles in Rust/WASM and passes existing unit/static tests, but has NOT been exercised against an actual enchanting table. Do not merge based on those checks alone.

It recalculates vanilla shelf offer ranges through enchantment_level_set when water blocks the power path. It uses level RNG rather than the player's enchantment seed; particles remain vanilla. The source reference is Serilum/Underwater-Enchanting, branch 1.21.11, EnchantingTableBlockMixin.java. No original Java code is incorporated.

Required next validation: disabled control, underwater 15-shelf third-row level 30, obstructed shelves rejected, actual enchantment/lapis/XP consumption, and ordinary dry-table behavior unchanged. The weekly quota reached the user's 5% stop threshold before live validation. The branch is not part of the approved tested merge.
