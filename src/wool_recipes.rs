//! Load-phase admission for optional Wool Tweaks recipes. Vanilla serializers
//! handle matching, consumption, remainders, recipe-book display and sync.
use crate::settings::on;
use mod_api::{ConditionError, Identifier, LoadCondition, RegisterLoadConditionsEvent, ServerHost};
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

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
    if let Ok(id) = Identifier::parse("horizon_qol_suite:wool_tweaks_enabled") {
        // The collector retains and reports registration refusals at load end.
        let _ = event.conditions.register(id, Enabled);
    }
}

pub fn refresh(host: &dyn ServerHost) {
    ENABLED.store(on(host, "wool_tweaks"), Ordering::Relaxed);
}

pub fn reset() {
    ENABLED.store(false, Ordering::Relaxed);
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
