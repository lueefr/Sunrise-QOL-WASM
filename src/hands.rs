//! First-person hand visibility through the renderer's public cancel hook.
use mod_api::{ClientHost, Flow, Hand, RenderHandEvent};
use crate::settings::{flag, on};

pub fn on_hand(host: &dyn ClientHost, event: &mut RenderHandEvent<'_, '_>) -> Flow {
    if !on(host, "hide_hands") { return Flow::Continue; }
    let main = matches!(event.render.hand, Hand::Main);
    if main {
        if flag(host, "alwaysHideMainHand", false)
            || (event.render.stack.is_none() && flag(host, "alwaysHideEmptyMainHand", true)) {
            return Flow::Cancel;
        }
    } else if flag(host, "alwaysHideOffhand", false) {
        return Flow::Cancel;
    } else if let Some(stack) = event.render.stack.as_ref() {
        let id = stack.item().id(host);
        if id == "minecraft:totem_of_undying" || id == "minecraft:torch" {
            return Flow::Cancel;
        }
    }
    Flow::Continue
}
