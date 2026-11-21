//! Load-phase admission for optional Wool Tweaks recipes. Vanilla serializers
//! handle matching, consumption, remainders, recipe-book display and sync.
use crate::settings::on;
use mod_api::{ConditionError, Identifier, LoadCondition, RegisterLoadConditionsEvent, ServerHost};
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static FURNACE: AtomicBool = AtomicBool::new(false);
static BELL: AtomicBool = AtomicBool::new(false);

struct OptionalRecipe(&'static AtomicBool);
impl LoadCondition for OptionalRecipe {
    fn test(&self, args: &serde_json::Value) -> Result<bool, ConditionError> {
        if !args.as_object().is_some_and(|object| object.is_empty()) {
            return Err(ConditionError::new("recipe condition takes no arguments"));
        }
        Ok(self.0.load(Ordering::Relaxed))
    }
}

struct Enabled;
impl LoadCondition for Enabled {
    fn test(&self, args: &serde_json::Value) -> Result<bool, ConditionError> {
        if !args.as_object().is_some_and(|object| object.is_empty()) {
            return Err(ConditionError::new("wool_tweaks_enabled takes no arguments"));
        }
        Ok(ENABLED.load(Ordering::Relaxed))
    }
}

pub fn register(event: &mut RegisterLoadConditionsEvent<'_>) {
    for (key, enabled) in [("furnace_recycle", &FURNACE), ("village_bell_recipe", &BELL)] {
        if let Ok(id) = Identifier::parse(&format!("horizon_qol_suite:{key}_enabled")) {
            let _ = event.conditions.register(id, OptionalRecipe(enabled));
        }
    }
    if let Ok(id) = Identifier::parse("horizon_qol_suite:wool_tweaks_enabled") {
        // The collector retains and reports registration refusals at load end.
        let _ = event.conditions.register(id, Enabled);
    }
}

pub fn refresh(host: &dyn ServerHost) {
    ENABLED.store(on(host, "wool_tweaks"), Ordering::Relaxed);
    FURNACE.store(on(host, "furnace_recycle"), Ordering::Relaxed);
    BELL.store(on(host, "village_bell_recipe"), Ordering::Relaxed);
}

pub fn reset() {
    ENABLED.store(false, Ordering::Relaxed);
    FURNACE.store(false, Ordering::Relaxed);
    BELL.store(false, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn condition_rejects_arguments_instead_of_silently_ignoring_them() {
        assert!(Enabled.test(&serde_json::json!({"enabled": true})).is_err());
        assert!(Enabled.test(&serde_json::json!(null)).is_err());
        assert!(Enabled.test(&serde_json::json!({})).is_ok());
    }
}
