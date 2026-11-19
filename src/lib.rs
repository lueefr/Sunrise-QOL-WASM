//! Horizon QoL, rebuilt around Sunrise's public sandbox API rather than native mixins.
//! Every subscribed callback has bounded work; unsupported modules remain catalogued,
//! never represented as a working toggle.
mod anvil;
mod drops;
mod decay;
mod doors;
mod automatic_doors;
mod beacon;
mod conduit;
mod spawner;
mod bottled_air;
mod help;
mod conduit_drowned;
mod painting;
mod dismount;
mod elytra;
mod compass;
mod clock;
mod zoom;
mod totem;
mod shulkers;
mod mending;
mod enchant;
mod durability;
mod unlimited;
mod superflat;
mod grief;
mod gui;
mod hands;
mod messages;
mod mining;
mod settings;
mod trading;

use mod_api::{
    AnvilUpdateEvent, BlockChangedEvent, LevelHost, ClientHost, EntityJoinLevelEvent, EntityLeaveLevelEvent,
    EntityMobGriefingEvent, Flow, Hooks, Host, HostSide,
    LootDropsEvent, PlayerDestroyedBlockEvent, PlayerLoggedInEvent, Registrar,
    RegistrationContext, RenderHandEvent, ServerHost, ServerStoppedEvent, ServerTickEvent,
    TradeWithVillagerEvent, InputKeyEvent, ClientTickPreEvent, ScreenRenderPostEvent,
    ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent,
    ScreenMouseDraggedPreEvent, ScreenKeyPressedPreEvent, ScreenCharTypedPreEvent,
    ScreenMouseScrolledPreEvent, ScreenRenderPreEvent, SettingChangedEvent,
    RightClickBlockEvent, RightClickItemEvent, GuiRenderPostEvent, LivingDeathEvent, LivingDeathPostEvent,
    PlayerXpChangeEvent, ItemTooltipEvent, MobSpawnPositionCheckEvent,
    PlayerCommandEvent, PlayerCommandsSentEvent, EntityInteractEvent, InteractionResult,
};

#[derive(Default)]
pub struct SunriseQol {
    mining: mining::Mining,
    decay: decay::Decay,
    villagers: grief::Villagers,
    gui: gui::Gui,
    doors: doors::Doors,
    automatic_doors: automatic_doors::AutomaticDoors,
    beacon: beacon::Beacon,
    conduit: conduit::Conduit,
    help: help::Help,
    elytra: elytra::Elytra,
    compass: compass::Compass,
    clock: clock::Clock,
    shulkers: shulkers::Shulkers,
}

#[mod_api::sunrise_mod]
impl Hooks for SunriseQol {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        settings::register(r);
        zoom::register(r);
        durability::register(r);
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        self.gui.setting_changed(host, event);
    }

    fn input_key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        self.gui.key(host, event);
    }

    fn client_tick_pre(&self, host: &mut dyn ClientHost, event: &mut ClientTickPreEvent) {
        self.gui.tick(host, event);
        self.elytra.tick(host, event);
    }

    fn screen_render_pre(&self, host: &dyn ClientHost, event: &mut ScreenRenderPreEvent<'_>) -> Flow {
        self.gui.render_pre(host, event)
    }

    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        self.gui.draw(host, event);
    }

    fn screen_mouse_scrolled_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        self.gui.wheel(host, event)
    }

    fn screen_mouse_button_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
        self.gui.pressed(host, event)
    }

    fn screen_mouse_button_released_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonReleasedPreEvent<'_>) -> Flow {
        self.gui.released(host, event)
    }

    fn screen_mouse_dragged_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseDraggedPreEvent<'_>) -> Flow {
        self.gui.dragged(host, event)
    }

    fn screen_key_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenKeyPressedPreEvent<'_>) -> Flow {
        self.gui.key_pressed(host, event)
    }

    fn screen_char_typed_pre(&self, host: &dyn ClientHost, event: &mut ScreenCharTypedPreEvent<'_>) -> Flow {
        self.gui.typed(host, event)
    }

    fn player_destroyed_block(&self, host: &mut dyn ServerHost, event: &mut PlayerDestroyedBlockEvent) {
        self.mining.block(host, event);
        self.beacon.broken(host, event);
        self.conduit.broken(host, event);
    }

    fn block_changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        self.decay.changed(level, event);
        self.doors.changed(level, event);
    }

    fn right_click_item(&self, side: HostSide<'_>, event: &mut RightClickItemEvent<'_>) -> Flow {
        match side {
            HostSide::Client(host) => self.elytra.clicked(HostSide::Client(host), event),
            HostSide::Server(host) => bottled_air::use_bottle(HostSide::Server(host), event),
        }
    }

    fn entity_interact(&self, side: HostSide<'_>, event: &mut EntityInteractEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = side else { return None; };
        if let Some(answer) = painting::interact(host, event) { return Some(answer); }
        dismount::interact(host, event)
    }

    fn right_click_block(&self, side: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        let HostSide::Server(host) = side else { return Flow::Continue; };
        if self.beacon.click(host, event) == Flow::Cancel { return Flow::Cancel; }
        if self.conduit.click(host, event) == Flow::Cancel { return Flow::Cancel; }
        self.doors.clicked(HostSide::Server(host), event)
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        self.compass.render(host, event);
        self.clock.render(host, event);
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        enchant::tooltip(host, event.stack, event.lines);
    }

    fn zoom(&self, host: &dyn ClientHost, event: &mut mod_api::ZoomEvent) -> Option<mod_api::Zoom> {
        zoom::current(host, event)
    }

    fn player_command(&self, host: &mut dyn ServerHost, event: &mut PlayerCommandEvent<'_>) -> Flow {
        self.help.command(host, event)
    }

    fn player_commands_sent(&self, host: &mut dyn ServerHost, event: &mut PlayerCommandsSentEvent<'_>) {
        self.help.sent(host, event);
    }

    fn mob_spawn_position_check(&self, host: &dyn Host, event: &mut MobSpawnPositionCheckEvent<'_>) {
        spawner::check(host, event);
        conduit_drowned::check(host, event);
    }

    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        self.decay.tick(host, event);
        self.doors.tick(host, event);
        self.automatic_doors.tick(host, event);
        self.shulkers.tick(host, event);
    }

    fn loot_drops(&self, host: &mut dyn ServerHost, event: &mut LootDropsEvent<'_>) {
        drops::on_loot(host, event);
    }

    fn anvil_update(&self, side: HostSide<'_>, event: &mut AnvilUpdateEvent<'_>) -> Flow {
        anvil::on_update(side, event)
    }

    fn player_xp_change(&self, host: &mut dyn ServerHost, event: &mut PlayerXpChangeEvent<'_>) -> Flow {
        mending::on_xp(host, event)
    }

    fn living_death(&self, host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
        self.shulkers.death(host, event);
        totem::on_death(host, event)
    }

    fn living_death_post(&self, host: &mut dyn ServerHost, event: &mut LivingDeathPostEvent<'_>) {
        self.shulkers.confirmed(host, event);
    }

    fn player_logged_in(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
        messages::on_join(host, event);
    }

    fn trade_with_villager(&self, host: &mut dyn ServerHost, event: &mut TradeWithVillagerEvent<'_>) {
        trading::on_trade(host, event);
    }

    fn entity_join_level(&self, host: &mut dyn ServerHost, event: &mut EntityJoinLevelEvent<'_>) -> Flow {
        if superflat::on_join(host, event) == Flow::Cancel { return Flow::Cancel; }
        self.villagers.on_join(event)
    }

    fn entity_leave_level(&self, _: &mut dyn ServerHost, event: &mut EntityLeaveLevelEvent<'_>) {
        self.villagers.on_leave(event);
    }

    fn entity_mob_griefing(&self, host: &dyn Host, event: &mut EntityMobGriefingEvent) {
        self.villagers.on_grief(host, event);
    }

    fn server_stopped(&self, _: &mut dyn ServerHost, _: &mut ServerStoppedEvent) {
        self.villagers.reset();
        self.decay.reset();
        self.doors.reset();
        self.automatic_doors.reset();
        self.help.reset();
        self.elytra.reset();
        self.shulkers.reset();
    }

    fn render_hand(&self, host: &dyn ClientHost, event: &mut RenderHandEvent<'_, '_>) -> Flow {
        hands::on_hand(host, event)
    }
}
