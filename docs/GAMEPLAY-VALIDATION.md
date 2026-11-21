# Optional manual gameplay validation

The **real-gameplay test is no longer part of CI**. It was removed from the
pull-request and release workflows to reduce build time and remove the
dependency on a private runner and prepared Minecraft resources.

CI now checks the Rust/Python source, `.smod` archive, binary ABI and
loading/initialization in Sunrise's real Wasmtime guest runtime. A green CI
result **does not** establish 1:1 gameplay parity.

The historical `tools/gameplay/` fixtures remain for manual local testing.
They are not automatically executed or required for publishing. Running
them requires legitimately prepared Minecraft game resources. Such private
assets must not be committed or uploaded to CI.

Test module behavior manually inside Sunrise before claiming gameplay
compatibility, including world state, multiplayer, GUI, renderer and the
original mod's edge cases.
