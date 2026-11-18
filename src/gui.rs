//! A single scrolling settings screen built with the public Sunrise WASM widget API.
//! A one-item Choice opens a custom view while Options remains logically active.
//! The Render.Pre hook hides the host page, and Render.Post draws the custom view.
//! F8 keeps the ordinary mod-screen route while a world is open.
use crate::settings::{amount, flag, on, save, saved_value, ID};
sue mod_api::client::widgets::{Button, Outcome, TextField, WidgetId, Widgets};
use mod_api::{
    ClientHost, ClientTickPreEvent, Flow, InputKeyEvent, KeyAction,
    ScreenCharTypedPreEvent, ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent,
    ScreenMouseButtonReleasedPreEvent, ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent,
    ScreenRenderPreEvent, ScreenRenderPostEvent, SettingChangedEvent, Text,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const SCREEN_ID: &str = "horizon_qol_suite";
const ROW_H: f32 = 25.0;
const CATEGORY_H: f32 = 32.0;
const PAD: f32 = 6.0;
const SCROLL_STEP: f32 = 26.0;

#[derive(Clone, Copy)]
enum OptionKind {
    Toggle { default: bool },
    Range { default: f64, min: f64, max: f64, step: f64 },
    Choice { default: &'static str, values: &'static [&'static str] },
}

#[derive(Clone, Copy)]
struct OptionSpec {
    key: &'static str,
    label: &'static str,
    kind: OptionKind,
}

#[derive(Clone, Copy)]
struct ModuleSpec {
    key: &'static str,
    label: &'static str,
    options: &'static [OptionSpec],
    category: Category,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    Enchanting,
    Inventory,
    Hud,
    Gathering,
    World,
    Creatures,
    Trading,
    Messages,
}

impl Category {
    const fn label(self) -> &'static str {
        match self {
            Self::Enchanting => "Enchanting & Anvils",
            Self::Inventory => "Inventory & Equipment",
            Self::Hud => "HUD & Camera",
            Self::Gathering => "Mining & Nature",
            Self::World => "World Interactions",
            Self::Creatures => "Mobs & Loot",
            Self::Trading => "Villager Trading",
            Self::Messages => "Player Messages",
        }
    }
}

const fn toggle(key: &'static str, label: &'static str, default: bool) -> OptionSpec {
    OptionSpec { key, label, kind: OptionKind::Toggle { default } }
}

const fn range(key: &'static str, label: &'static str, default: f64, min: f64, max: f64, step: f64) -> OptionSpec {
    OptionSpec { key, label, kind: OptionKind::Range { default, min, max, step } }
}

const fn choice(key: &'static str, label: &'static str, default: &'static str,
                values: &'static [&'static str]) -> OptionSpec {
    OptionSpec { key, label, kind: OptionKind::Choice { default, values } }
}

const DOORS: &[OptionSpec] = &[
    toggle("recursive_doors", "Recursive Opening", true),
    range("door_radius", "Maximum Search Distance", 10., 1., 64., 1.),
    toggle("gates", "Fence Gates", false),
    toggle("trapdoors", "Trapdoors", false),
];

const COMPASS: &[OptionSpec] = &[
    choice("guiCompassFormat", "Display", "FXYZ", &["FXYZ", "F", "XYZ"]),
    toggle("mustHaveCompassInInventory", "Require Compass in Inventory", true),
    choice("compassPosition", "Position", "Left", &["Left", "Center", "Right"]),
    range("compassHeightOffset", "Top Offset", 5., 0., 3000., 5.),
    toggle("drawTextShadow", "Text Shadow", true),
    range("compass_red", "Text Color: Red", 255., 0., 255., 5.),
    range("compass_green", "Text Color: Green", 255., 0., 255., 5.),
    range("compass_blue", "Text Color: Blue", 255., 0., 255., 5.),
];

const CLOCK: &[OptionSpec] = &[
    toggle("mustHaveClockInInventoryForGameTime", "Require Clock in Inventory", true),
    toggle("clock24Hour", "24-Hour Format", true),
    toggle("showDaysPlayedWorld", "Show Days Played", true),
    range("minuteStep", "Minute Precision", 1., 1., 60., 1.),
    choice("clockPosition", "Position", "Right", &["Left", "Center", "Right"]),
    range("clockHeightOffset", "Top Offset", 5., 0., 3000., 5.),
    range("clockWidthOffset", "Horizontal Offset", 0., -3000., 3000., 5.),
    toggle("clockTextShadow", "Text Shadow", true),
    range("clock_red", "Text Color: Red", 255., 0., 255., 5.),
    range("clock_green", "Text Color: Green", 255., 0., 255., 5.),
    range("clock_blue", "Text Color: Blue", 255., 0., 255., 5.),
];

const MENDING: &[OptionSpec] = &[
    toggle("mending_hotbar_only", "Repair Hotbar Only", false),
];

const RESPAWN: &[OptionSpec] = &[
    range("respawn_ticks", "Respawn Delay (Ticks)", 1200., 1., 72000., 20.),
    toggle("skip_spawner", "Ignore Spawner Shulkers", true),
];

const ZOOM: &[OptionSpec] = &[
    range("zoom_factor", "Zoom Factor", 10., 2., 30., 1.),
    toggle("zoom_smooth", "Smooth Camera Zoom", true),
];

const ENCHANT: &[OptionSpec] = &[
    choice("cap_symbol", "Maximum Level Symbol", "★", &["★", "✦", "*"]),
    range("cap_color", "Symbol Color", 6., 0., 15., 1.),
];

const SUPERFLAT: &[OptionSpec] = &[
    toggle("confirm_superflat", "Confirm This Is a Superflat World", false),
];

const TAXFREE: &[OptionSpec] = &[
    toggle("taxfree_cheap_rename", "One-Level Renaming", true),
    toggle("taxfree_remove_anvil_limit", "Remove Too Expensive Limit", true),
];

const UNLIMITED: &[OptionSpec] = &[
    range("unlimited_max_level", "Maximum Enchantment Level", 255., 1., 255., 1.),
    range("unlimited_max_anvil_cost", "Maximum Anvil Cost", 39., 1., 10000., 1.),
];

const LANGPATCH: &[OptionSpec] = &[
    toggle("patch_potions", "Patch Potion Potency", true),
    toggle("patch_override", "Override Existing Translations", false),
    toggle("enchant_roman", "Use Roman Enchantment Levels", false),
    toggle("potion_roman", "Use Roman Potion Levels", false),
];

const TREE: &[OptionSpec] = &[
    toggle("mustHoldAxeForTreeHarvest", "Require an Axe", true),
    toggle("treeHarvestWithoutCrouching", "Harvest Without Sneaking", false),
    toggle("treeHarvestIgnoresCrouching", "Ignore Sneaking", false),
    toggle("automaticallyFindBottomBlock", "Find Bottom Log Automatically", true),
    toggle("ignorePlayerMadeTrees", "Ignore Player-Planted Trees", true),
    toggle("enableNetherTrees", "Nether Trees", true),
    toggle("enableHugeMushrooms", "Huge Mushrooms", true),
    range("maxAmountOfLogsBrokenPerHarvest", "Maximum Logs", 256., 1., 1024., 16.),
];

const LEAF_DECAY: &[OptionSpec] = &[
    toggle("accelerateLeafDecay", "Accelerated Leaf Decay", true),
    range("minimumDecayTicks", "Minimum Decay Ticks", 5., 0., 1200., 1.),
    range("maximumDecayTicks", "Maximum Decay Ticks", 20., 0., 1200., 1.),
    range("amountOfLeavesBrokenPerTick", "Leaves per Tick", 5., 1., 64., 1.),
    toggle("matchLeafTypes", "Match Leaf Types", true),
    toggle("unknownLeavesMatchSelf", "Unknown Leaves Match Self", true),
    toggle("matchLogsToLeaves", "Match Logs to Leaves", true),
    toggle("woodPreventsDecay", "Wood Prevents Decay", false),
    toggle("decayEffects", "Decay Sound and Particles", false),
    range("persistentLeavesBehavior", "Persistent Leaves Mode", 0., 0., 2., 1.),
];

const VEIN: &[OptionSpec] = &[
    toggle("oreHarvestWithoutSneak", "Mine Without Sneaking", false),
    toggle("enableFuzzyOreSearch", "Diagonal Ore Search", true),
    range("maxOresPerVein", "Maximum Ores", 256., 1., 1024., 16.),
];

const ANVIL: &[OptionSpec] = &[
    toggle("affordable_remove_too_expensive", "Remove Too Expensive Limit", true),
    toggle("affordable_cheap_rename", "One-Level Renaming", false),
    range("affordable_cost_cap", "Maximum Anvil Cost", 39., 1., 10000., 1.),
];

const SHULKER: &[OptionSpec] = &[
    toggle("always_drop", "Always Drop Shells", false),
    range("drop_count", "Shells per Drop", 2., 1., 64., 1.),
];

const TRADING: &[OptionSpec] = &[
    toggle("villagers", "Villagers", true),
    toggle("wandering_trader", "Wandering Trader", true),
    toggle("remove_demand", "Remove Demand", true),
];

const HANDS: &[OptionSpec] = &[
    toggle("alwaysHideMainHand", "Hide Main Hand", false),
    toggle("alwaysHideEmptyMainHand", "Hide Empty Main Hand", true),
    toggle("alwaysHideOffhand", "Hide Offhand", false),
];

// Only modules with actual hook implementations are listed.
// Enchantment extraction will join Enchanting when a working hook is available.
const MODULES: &[ModuleSpec] = &[
    ModuleSpec { key: "unlimited_enchantments", label: "Enchantment Levels", options: UNLIMITED, category: Category::Enchanting },
    ModuleSpec { key: "affordable_anvil", label: "Anvil Costs", options: ANVIL, category: Category::Enchanting },
    ModuleSpec { key: "tax_free_levels", label: "XP Cost Rules", options: TAXFREE, category: Category::Enchanting },
    ModuleSpec { key: "enchantment_level_indicator", label: "Max-Level Indicators", options: ENCHANT, category: Category::Enchanting },
    ModuleSpec { key: "ench_level_langpatch", label: "Level Formatting", options: LANGPATCH, category: Category::Enchanting },
    ModuleSpec { key: "inventory_totem", label: "Inventory Totems", options: &[], category: Category::Inventory },
    ModuleSpec { key: "inventory_mending", label: "XP Item Repair", options: MENDING, category: Category::Inventory },
    ModuleSpec { key: "what_durability", label: "Durability Control", options: &[], category: Category::Inventory },
    ModuleSpec { key: "gui_clock", label: "World Clock", options: CLOCK, category: Category::Hud },
    ModuleSpec { key: "gui_compass", label: "Navigation Compass", options: COMPASS, category: Category::Hud },
    ModuleSpec { key: "vanilla_zoom", label: "Camera Zoom", options: ZOOM, category: Category::Hud },
    ModuleSpec { key: "hide_hands", label: "Hand Visibility", options: HANDS, category: Category::Hud },
    ModuleSpec { key: "vein_miner", label: "Vein Mining", options: VEIN, category: Category::Gathering },
    ModuleSpec { key: "tree_harvester", label: "Tree Harvesting", options: TREE, category: Category::Gathering },
    ModuleSpec { key: "leaf_decay", label: "Leaf Decay", options: LEAF_DECAY, category: Category::Gathering },
    ModuleSpec { key: "double_doors", label: "Paired Doors", options: DOORS, category: Category::World },
    ModuleSpec { key: "friendly_griefing", label: "Mob Griefing", options: &[], category: Category::World },
    ModuleSpec { key: "superflat_world_no_slimes", label: "Superflat Slime Spawns", options: SUPERFLAT, category: Category::World },
    ModuleSpec { key: "shulker_drops_two", label: "Shulker Shell Drops", options: SHULKER, category: Category::Creatures },
    ModuleSpec { key: "respawning_shulkers", label: "Shulker Respawning", options: RESPAWN, category: Category::Creatures },
    ModuleSpec { key: "always_wither_skull", label: "Wither Skull Drops", options: &[], category: Category::Creatures },
    ModuleSpec { key: "dragon_drops_elytra", label: "Dragon Drops", options: &[], category: Category::Creatures },
    ModuleSpec { key: "infinite_trading", label: "Trade Restocking", options: TRADING, category: Category::Trading },
    ModuleSpec { key: "first_join_message", label: "First Join Message", options: &[], category: Category::Messages },
    ModuleSpec { key: "welcome_message", label: "Welcome Messages", options: &[], category: Category::Messages },
];
#[derive(Clone, Copy)]
enum Action {
    Expand(usize),
    Switch(&'static str, bool),
    Adjust(OptionSpec, i8),
    Cycle(OptionSpec),
}

struct Label {
    text: String,
    at: [f32; 2],
    color: u32,
}

struct Screen {
    widgets: Widgets,
    footer: Widgets,
    actions: Vec<(WidgetId, Action)>,
    numbers: Vec<(WidgetId, OptionSpec)>,
    labels: Vec<Label>,
    expanded: Option<usize>,
    size: [f32; 2],
    viewport: [f32; 4],
    scroll: f32,
    total: f32,
    drag_bar: bool,
}

impl Screen {
    fn option_amount(&self, host: &dyn ClientHost, key: &str,
                     default: f64, min: f64, max: f64) -> f64 {
        let component = match key {
            "compass_red" => Some(0), "compass_green" => Some(1),
            "compass_blue" => Some(2), _ => None,
        };
        if let Some(index) = component {
            if saved_value(host, key).is_none() {
                if let Some(rgb) = saved_value(host, "compassRGB") {
                    if let Some(value) = rgb.split(',').nth(index).and_then(|c| c.trim().parse::<f64>().ok()) {
                        return value.clamp(min, max);
                    }
                }
            }
        }
        amount(host, key, default, min, max)
    }

    fn label(&mut self, text: impl Into<String>, x: f32, y: f32, color: u32) {
        self.labels.push(Label { text: text.into(), at: [x, y], color });
    }

    fn button(&mut self, rect: [f32; 4], name: &str, enabled: bool, action: Action) {
        if !self.visible(rect[1], rect[3]) {
            return;
        }
        let mut button = Button::new(rect, Text::literal(name));
        button.active = enabled;
        let id = self.widgets.add_button(button);
        self.actions.push((id, action));
    }

    fn number(&mut self, rect: [f32; 4], option: OptionSpec, value: f64, active: bool) {
        if !self.visible(rect[1], rect[3]) { return; }
        let mut field = TextField::new(rect, 16);
        field.set_value(&format!("{value:.0}"));
        field.message = option.label.to_owned();
        field.active = active;
        let id = self.widgets.add_text_field(field);
        self.numbers.push((id, option));
    }

    fn valid_number(value: &str, min: f64, max: f64) -> bool {
        value.trim().parse::<f64>().ok()
            .is_some_and(|number| number.is_finite() && number >= min && number <= max)
    }

    fn mark_number(&mut self, id: WidgetId) {
        let Some((_, option)) = self.numbers.iter().find(|(key, _)| *key == id).copied() else {
            return;
        };
        let OptionKind::Range { min, max, .. } = option.kind else { return; };
        let Some(field) = self.widgets.text_field_mut(id) else { return; };
        field.text_color = if Self::valid_number(field.value(), min, max) {
            0xFFE0_E0E0
        } else {
            0xFFFF_7777
        };
    }

    // Keep partial edits on screen, but store only a complete finite number.
    fn commit_number(&mut self, host: &dyn ClientHost, id: WidgetId) {
        let Some((_, option)) = self.numbers.iter().find(|(key, _)| *key == id).copied() else {
            return;
        };
        let OptionKind::Range { default, min, max, .. } = option.kind else { return; };
        let old = self.option_amount(host, option.key, default, min, max);
        let draft = self.widgets.text_field(id).map(|field| field.value().trim().to_owned());
        let value = draft.as_deref().and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite());
        let shown = if let Some(value) = value {
            let bounded = value.round().clamp(min, max);
            if bounded == old || save(host, option.key, &format!("{bounded:.0}")) {
                bounded
            } else {
                old
            }
        } else {
            old
        };
        if let Some(field) = self.widgets.text_field_mut(id) {
            field.set_value(&format!("{shown:.0}"));
            field.text_color = 0xFFE0_E0E0;
        }
    }

    fn commit_focused(&mut self, host: &dyn ClientHost) {
        if let Some(id) = self.widgets.focused() {
            self.commit_number(host, id);
        }
    }

    fn visible(&self, y: f32, height: f32) -> bool {
        y >= self.viewport[1] && y + height <= self.viewport[1] + self.viewport[3]
    }

    fn inside(&self, at: [f32; 2]) -> bool {
        let [x, y, w, h] = self.viewport;
        at[0] >= x && at[0] < x + w && at[1] >= y && at[1] < y + h
    }

    fn scrollbar(&self) -> Option<[f32; 4]> {
        if self.total <= self.viewport[3] { return None; }
        let [x, y, w, h] = self.viewport;
        let height = (h * h / self.total).clamp(20., h);
        let travel = (h - height).max(0.);
        let offset = self.scroll / self.max_scroll() * travel;
        Some([x + w - 7., y + offset, 6., height])
    }

    fn max_scroll(&self) -> f32 {
        (self.total - self.viewport[3]).max(0.)
    }

    fn row(&mut self, host: &dyn ClientHost, index: usize, x: f32, y: f32, w: f32) -> f32 {
        let module = &MODULES[index];
        let open = self.expanded == Some(index);
        let active = on(host, module.key);
        self.button([x, y, w - 94., 20.], " ", true, Action::Expand(index));
        if self.visible(y, 20.) {
            self.label(format!("{} {}", if open { "v" } else { ">" }, module.label),
                x + 8., y + 6., 0xFFFF_FFFF);
        }
        self.button([x + w - 88., y, 82., 20.], if active { "ON" } else { "OFF" }, true,
            Action::Switch(module.key, false));
        let mut next = y + ROW_H;
        if open {
            if module.options.is_empty() {
                if self.visible(next, 16.) {
                    self.label("No Additional Options", x + 15., next + 3., 0xFFAA_AAAA);
                }
                next += ROW_H;
            } else {
                for opt in module.options {
                    if self.visible(next, 20.) {
                        self.label(opt.label, x + 15., next + 6., 0xFFE0_E0E0);
                        match opt.kind {
                            OptionKind::Toggle { default } => {
                                let enabled = flag(host, opt.key, default);
                                self.button([x + w - 88., next, 82., 20.],
                                    if enabled { "ON" } else { "OFF" }, active,
                                    Action::Switch(opt.key, default));
                            }
                            OptionKind::Range { default, min, max, .. } => {
                                let value = self.option_amount(host, opt.key, default, min, max);
                                self.button([x + w - 114., next, 23., 20.], "-", active && value > min,
                                    Action::Adjust(*opt, -1));
                                let printed = if opt.key == "persistentLeavesBehavior" {
                                    match value as usize {
                                        0 => "IGNORE".to_owned(),
                                        1 => "NORMAL".to_owned(),
                                        _ => "ALL".to_owned(),
                                    }
                                } else { format!("{value:.0}") };
                                if opt.key == "persistentLeavesBehavior" {
                                    self.label(printed, x + w - 84., next + 6., 0xFFFF_FFFF);
                                } else {
                                    self.number([x + w - 88., next, 56., 20.], *opt, value, active);
                                }
                                self.button([x + w - 29., next, 23., 20.], "+", active && value < max,
                                    Action::Adjust(*opt, 1));
                            }
                            OptionKind::Choice { default, values } => {
                                let current = saved_value(host, opt.key).unwrap_or_else(|| default.to_owned());
                                let current = if values.contains(&current.as_str()) { current } else { default.to_owned() };
                                self.button([x + w - 151., next, 145., 20.], &current, active, Action::Cycle(*opt));
                            }
                        }
                    }
                    next += ROW_H;
                }
            }
        }
        next
    }

    fn new(host: &dyn ClientHost, size: [f32; 2], scroll: f32, expanded: Option<usize>) -> Self {
        let w = (size[0] - 28.).min(490.).max(100.);
        let x = (size[0] - w) / 2.;
        let top = 37.;
        let bottom = (size[1] - 34.).max(top + 24.);
        let viewport = [x, top, w, bottom - top];
        let total = PAD * 2. + MODULES.iter().enumerate().map(|(index, module)| {
            let header = if index == 0 || MODULES[index - 1].category != module.category {
                CATEGORY_H
            } else { 0. };
            header + ROW_H + if expanded == Some(index) {
                ROW_H * module.options.len().max(1) as f32
            } else { 0. }
        }).sum::<f32>();
        let scroll = scroll.clamp(0., (total - viewport[3]).max(0.));
        let mut screen = Self {
            widgets: Widgets::new(), footer: Widgets::new(), actions: Vec::new(),
            numbers: Vec::new(), labels: Vec::new(),
            expanded, size, viewport, scroll, total, drag_bar: false,
        };
        let mut y = top + PAD - scroll;
        for index in 0..MODULES.len() {
            let category = MODULES[index].category;
            if index == 0 || MODULES[index - 1].category != category {
                if screen.visible(y, 21.) {
                    screen.label(category.label(), x + 9., y + 8., 0xFFFF_DDA0);
                }
                y += CATEGORY_H;
            }
            y = screen.row(host, index, x + 12., y, w - 23.);
        }
        screen.footer.add_button(Button::new(
            [size[0] / 2. - 60., size[1] - 27., 120., 20.], Text::literal("Done")
        ));
        screen
    }

    fn at_bar(&self, point: [f32; 2]) -> bool {
        self.scrollbar().is_some_and(|bar| {
            point[0] >= bar[0] - 3. && point[0] < bar[0] + bar[2] + 3.
                && point[1] >= self.viewport[1] && point[1] < self.viewport[1] + self.viewport[3]
        })
    }

    fn move_bar(&self, point: [f32; 2]) -> f32 {
        let Some(bar) = self.scrollbar() else { return 0.; };
        let height = (self.viewport[3] - bar[3]).max(1.);
        let relative = (point[1] - self.viewport[1] - bar[3] / 2.).clamp(0., height);
        relative / height * self.max_scroll()
    }
}

#[derive(Default)]
pub struct Gui {
    screen: Mutex<Option<Screen>>,
    close: AtomicBool,
    overlay: AtomicBool,
}

impl Gui {
    fn active(&self, host: &dyn ClientHost, screen_id: &str) -> bool {
        host.mod_screen() == Some(SCREEN_ID)
            || (screen_id == "options" && self.overlay.load(Ordering::Relaxed))
    }

    fn in_overlay(&self, screen_id: &str) -> bool {
        screen_id == "options" && self.overlay.load(Ordering::Relaxed)
    }

    // The owner-only hook is fired by this mod's Config page, not any other mod's.
    pub fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID && event.key == "open_ui"
            && host.observed_screen_id().is_some_and(|screen| screen.id == "options")
        {
            *self.screen.lock().unwrap_or_else(|error| error.into_inner()) = None;
            self.overlay.store(true, Ordering::Relaxed);
        }
    }

    pub fn key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        if event.action == KeyAction::Press && event.key == "key.keyboard.f8" && host.playing() {
            *self.screen.lock().unwrap_or_else(|error| error.into_inner()) = None;
            host.mod_screen_open(SCREEN_ID);
        }
    }

    pub fn tick(&self, host: &mut dyn ClientHost, _: &mut ClientTickPreEvent) {
        if self.close.swap(false, Ordering::Relaxed) {
            host.mod_screen_close(SCREEN_ID);
        }
        // Never leave an overlay armed after Options has been closed.
        if self.overlay.load(Ordering::Relaxed)
            && !host.observed_screen_id().is_some_and(|screen| screen.id == "options")
        {
            self.overlay.store(false, Ordering::Relaxed);
            *self.screen.lock().unwrap_or_else(|error| error.into_inner()) = None;
        }
    }

    // The Options page remains the active host page, but is not painted while
    // the suite view is open. ScreenRenderPost still fires when Pre cancels.
    pub fn render_pre(&self, host: &dyn ClientHost, event: &mut ScreenRenderPreEvent<'_>) -> Flow {
        if self.in_overlay(event.screen.id)
            || (event.screen.id == "mod" && host.mod_screen() == Some(SCREEN_ID))
        {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    pub fn draw(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        if !self.active(host, event.screen.id) { return; }
        let size = event.draw.size();
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        if lock.as_ref().is_none_or(|screen| screen.size != size) {
            let (scroll, expanded) = lock.as_ref().map_or((0., None), |s| (s.scroll, s.expanded));
            if let Some(screen) = lock.as_mut() {
                screen.commit_focused(host);
            }
            *lock = Some(Screen::new(host, size, scroll, expanded));
        }
        let Some(screen) = lock.as_mut() else { return; };

        // A local player stays available with a screen open; playing() does not.
        // Reuse the same background sprites as Sunrise's native list screens.
        let in_world = host.local_player().is_some();
        let _ = host.apply_screen_blur();
        event.draw.draw_menu_background([0., 0., size[0], size[1]], in_world);
        let body = [0., 34., size[0], (size[1] - 68.).max(0.)];
        let sprite = if in_world {
            "gui/inworld_menu_list_background"
        } else {
            "gui/menu_list_background"
        };
        if !event.draw.tiled_sprite(sprite, body, 32., 0xFFFF_FFFF) {
            event.draw.fill(body, if in_world { 0x8800_0000 } else { 0xA000_0000 });
        }
        // Native-style list borders. The content is clipped to the central panel.
        event.draw.fill([0., 32., size[0], 1.], 0xFF55_5555);
        event.draw.fill([0., size[1] - 34., size[0], 1.], 0xFF55_5555);
        let title = "Quality of Life Settings";
        let x = ((size[0] - event.draw.text_width(title)) / 2.).max(0.);
        event.draw.text(title, [x, 15.], 0xFFFF_FFFF, true);
        let _ = event.draw.push_clip(screen.viewport);
        let _ = screen.widgets.draw(&mut *event.draw, host, event.screen.mouse);
        for label in &screen.labels {
            event.draw.text(&label.text, label.at, label.color, true);
        }
        let _ = event.draw.pop_clip();

        // Use the current scrollbar textures, falling back to plain rectangles.
        if let Some(bar) = screen.scrollbar() {
            let track = [bar[0], screen.viewport[1], 6., screen.viewport[3]];
            if !event.draw.sprite("widget/scroller_background", [0., 0., 6., 6.], track, None) {
                event.draw.fill(track, 0x8040_4040);
            }
            if !event.draw.sprite("widget/scroller", [0., 0., 6., 6.], bar, None) {
                event.draw.fill(bar, 0xFFB0_B0B0);
            }
        }
        // Keep the footer outside the scrolled and clipped area.
        let _ = screen.footer.draw(&mut *event.draw, host, event.screen.mouse);
    }

    fn input(&self, host: &dyn ClientHost, screen_id: &str, input: impl FnOnce(&mut Widgets) -> Outcome) -> Flow {
        if !self.active(host, screen_id) { return Flow::Continue; }
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        let Some(screen) = lock.as_mut() else {
            return if self.in_overlay(screen_id) { Flow::Cancel } else { Flow::Continue };
        };
        let before = screen.widgets.focused();
        let outcome = input(&mut screen.widgets);
        if before != screen.widgets.focused() {
            if let Some(id) = before {
                screen.commit_number(host, id);
            }
        }
        match outcome {
            Outcome::Changed(id) => screen.mark_number(id),
            Outcome::Submitted(id) => {
                screen.commit_number(host, id);
                screen.widgets.set_focus(host, None);
            }
            Outcome::Pressed(id) => {
                let action = screen.actions.iter().find(|(button, _)| *button == id)
                    .map(|(_, action)| *action);
                if let Some(action) = action {
                    let mut expanded = screen.expanded;
                    let size = screen.size;
                    let scroll = screen.scroll;
                    match action {
                        Action::Expand(index) => {
                            expanded = if expanded == Some(index) { None } else { Some(index) };
                        }
                        Action::Switch(key, default) => {
                            let current = flag(host, key, default);
                            let _ = save(host, key, if current { "false" } else { "true" });
                        }
                        Action::Cycle(option) => {
                            if let OptionKind::Choice { default, values } = option.kind {
                                let current = saved_value(host, option.key).unwrap_or_else(|| default.to_owned());
                                let index = values.iter().position(|value| *value == current).unwrap_or(0);
                                let _ = save(host, option.key, values[(index + 1) % values.len()]);
                            }
                        }
                        Action::Adjust(option, direction) => {
                            if let OptionKind::Range { default, min, max, step } = option.kind {
                                let current = screen.option_amount(host, option.key, default, min, max);
                                let next = (current + f64::from(direction) * step).clamp(min, max);
                                let _ = save(host, option.key, &format!("{next}"));
                            }
                        }
                    }
                    *screen = Screen::new(host, size, scroll, expanded);
                }
            }
            _ => {}
        }
        if outcome.handled() || self.in_overlay(screen_id) { Flow::Cancel } else { Flow::Continue }
    }

    pub fn wheel(&self, host: &dyn ClientHost, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        if !self.active(host, event.screen.id) { return Flow::Continue; }
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        let Some(screen) = lock.as_mut() else { return Flow::Cancel; };
        if !screen.inside(event.screen.mouse) {
            return if self.in_overlay(event.screen.id) { Flow::Cancel } else { Flow::Continue };
        }
        let next = screen.scroll - event.scroll[1] as f32 * SCROLL_STEP;
        screen.commit_focused(host);
        let size = screen.size;
        let expanded = screen.expanded;
        *screen = Screen::new(host, size, next, expanded);
        Flow::Cancel
    }

    pub fn pressed(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
        if !self.active(host, event.screen.id) { return Flow::Continue; }
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(screen) = lock.as_mut() {
            if event.button == 0 && screen.at_bar(event.screen.mouse) {
                screen.commit_focused(host);
                *screen = Screen::new(host, screen.size, screen.move_bar(event.screen.mouse), screen.expanded);
                screen.drag_bar = true;
                return Flow::Cancel;
            }
        }
        if let Some(screen) = lock.as_mut() {
            if screen.footer.mouse_pressed(host, event.screen.mouse, event.button, event.modifiers).handled() {
                screen.commit_focused(host);
                if self.in_overlay(event.screen.id) {
                    self.overlay.store(false, Ordering::Relaxed);
                    *lock = None;
                } else {
                    self.close.store(true, Ordering::Relaxed);
                }
                return Flow::Cancel;
            }
        }
        drop(lock);
        self.input(host, event.screen.id, |widgets| widgets.mouse_pressed(host, event.screen.mouse, event.button, event.modifiers))
    }

    pub fn released(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonReleasedPreEvent<'_>) -> Flow {
        if !self.active(host, event.screen.id) { return Flow::Continue; }
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(screen) = lock.as_mut() {
            if event.button == 0 && screen.drag_bar {
                screen.drag_bar = false;
                return Flow::Cancel;
            }
        }
        if let Some(screen) = lock.as_mut() {
            if screen.footer.mouse_released(event.button).handled() {
                return Flow::Cancel;
            }
        }
        drop(lock);
        self.input(host, event.screen.id, |widgets| widgets.mouse_released(event.button))
    }

    pub fn dragged(&self, host: &dyn ClientHost, event: &mut ScreenMouseDraggedPreEvent<'_>) -> Flow {
        if !self.active(host, event.screen.id) { return Flow::Continue; }
        let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(screen) = lock.as_mut() {
            if screen.drag_bar && event.button == 0 {
                screen.commit_focused(host);
                *screen = Screen::new(host, screen.size, screen.move_bar(event.screen.mouse), screen.expanded);
                screen.drag_bar = true;
                return Flow::Cancel;
            }
        }
        drop(lock);
        self.input(host, event.screen.id, |widgets| widgets.mouse_dragged(host, event.screen.mouse, event.button))
    }

    pub fn key_pressed(&self, host: &dyn ClientHost, event: &mut ScreenKeyPressedPreEvent<'_>) -> Flow {
        if self.active(host, event.screen.id) && event.key == "key.keyboard.escape" {
            let mut lock = self.screen.lock().unwrap_or_else(|error| error.into_inner());
            if let Some(screen) = lock.as_mut() {
                screen.commit_focused(host);
            }
            if self.in_overlay(event.screen.id) {
                self.overlay.store(false, Ordering::Relaxed);
                *lock = None;
                return Flow::Cancel;
            }
            return Flow::Continue;
        }
        self.input(host, event.screen.id, |widgets| widgets.key_pressed(host, event.key, event.modifiers))
    }

    pub fn typed(&self, host: &dyn ClientHost, event: &mut ScreenCharTypedPreEvent<'_>) -> Flow {
        self.input(host, event.screen.id, |widgets| widgets.char_typed(host, event.c))
    }
}
