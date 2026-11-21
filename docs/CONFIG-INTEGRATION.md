# Opening Sunrise QOL settings from the Sunrise Config page

The current implementation uses public Sunrise hooks.

1. Register `SettingSpec::choice("open_ui", "qol.config.open", ["qol.config.open.value", "qol.config.open.value"], 0)`.
2. The two identical labels alternate the saved choice index so Sunrise fires `setting_changed` on each click.
3. In the callback, require `event.mod_id == "sunrise_qol"`, `event.key == "open_ui"`, and `host.observed_screen_id().id == "options"`, then activate an overlay flag.
4. `screen_render_pre` returns `Flow::Cancel` when the Options overlay is active or the mod owns the opened `mod` screen. This prevents the underlying Options screen from drawing. The Sunrise API explicitly guarantees that `screen_render_post` still fires after the Pre hook cancels.
5. `screen_render_post` draws the suite's own full-screen menu background, native darkened list panel, header, widgets, and footer with the existing `DrawContext`. The correct menu textures are selected with `host.local_player().is_some()`, not `host.playing()`, which is false while a screen is open. Input hooks cancel mouse, scroll, and keyboard input while the overlay is active. Escape and Done close only the overlay.
6. Clear the overlay flag when Options is no longer visible to prevent accidental reopening.
7. F8 continues to use `mod_screen_open` within a world and is not used on top of Options.

## Where the other settings went

The Sunrise Config page displays every setting registered with `SettingSpec`. Therefore only `open_ui` is now registered. The module states and option values are stored in `config/sunrise_qol/<key>` via the owner-scoped `Host::config_files()` API; they cannot be written with the native `set_setting()` API once unregistered. The installer migrates settings from `sunrise-mods/.sunrise-mod-data/horizon_qol_suite/<key>` and `config/horizon_qol_suite/<key>` without overwriting `config/sunrise_qol/<key>`. The old data is retained as a rollback backup. Neither main-menu nor in-world menu should duplicate any toggles in the native page.

## Limitations

This is a **visual replacement only**, not a distinct registered Sunrise page; the Options page still owns navigation/focus underneath the canceled draw, and gamepad navigation is not trapped by mouse/keyboard hooks. This does **not** intercept the original Config click. It introduces a separate entry on the native Config page and therefore requires an additional click. Support from the in-game title menu still needs to be validated; the pre-Game startup menu may ignore mod-setting events. Gamepad input is not intercepted by the current overlay.

The build and WASM loader are checked in CI; real in-game behavior remains unverified.
