//! Settings backed by Sunrise's own per-mod storage, never by global mutable maps.
use mod_api::{Host, Registrar, SettingSpec};

pub const ID: &str = "horizon_qol_suite";

pub fn storage_key(key: &str) -> &str {
    match key {
        "clock24Hour" => "clock24hour",
        "clockPosition" => "clockposition",
        "clockHeightOffset" => "clockheightoffset",
        "clockWidthOffset" => "clockwidthoffset",
        "clockTextShadow" => "clocktextshadow",
        "minuteStep" => "minutestep",
        "minimumVillagerLevel" => "minimumvillagerlevel",
        "hungerLevelIncrease" => "hungerlevelincrease",
        "saturationLevelIncrease" => "saturationlevelincrease",
        "rchRequireHoe" => "rchrequirehoe",
        "rchHarvestInRadius" => "rchharvestinradius",
        "rchHungerLevel" => "rchhungerlevel",
        "rchExperienceType" => "rchexperiencetype",
        "showDaysPlayedWorld" => "showdaysplayedworld",
        "mustHaveClockInInventoryForGameTime" => "musthaveclockininventoryforgametime",
        "alwaysHideEmptyMainHand" => "alwayshideemptymainhand",
        "accelerateLeafDecay" => "accelerateleafdecay",
        "minimumDecayTicks" => "minimumdecayticks",
        "maximumDecayTicks" => "maximumdecayticks",
        "matchLeafTypes" => "matchleaftypes",
        "unknownLeavesMatchSelf" => "unknownleavesmatchself",
        "matchLogsToLeaves" => "matchlogstoleaves",
        "persistentLeavesBehavior" => "persistentleavesbehavior",
        "woodPreventsDecay" => "woodpreventsdecay",
        "decayEffects" => "decayeffects",
        "guiCompassFormat" => "guicompassformat",
        "mustHaveCompassInInventory" => "musthavecompassininventory",
        "compassPosition" => "compassposition",
        "compassHeightOffset" => "compassheightoffset",
        "drawTextShadow" => "drawtextshadow",
        "compassRGB" => "compassrgb",
        "alwaysHideMainHand" => "alwayshidemainhand",
        "alwaysHideOffhand" => "alwayshideoffhand",
        "amountOfLeavesBrokenPerTick" => "amountofleavesbrokenpertick",
        "automaticallyFindBottomBlock" => "automaticallyfindbottomblock",
        "enableFastLeafDecay" => "enablefastleafdecay",
        "enableFuzzyOreSearch" => "enablefuzzyoresearch",
        "enableHugeMushrooms" => "enablehugemushrooms",
        "enableNetherTrees" => "enablenethertrees",
        "ignorePlayerMadeTrees" => "ignoreplayermadetrees",
        "maxAmountOfLogsBrokenPerHarvest" => "maxamountoflogsbrokenperharvest",
        "maxOresPerVein" => "maxorespervein",
        "mustHoldAxeForTreeHarvest" => "mustholdaxefortreeharvest",
        "oreHarvestWithoutSneak" => "oreharvestwithoutsneak",
        "treeHarvestIgnoresCrouching" => "treeharvestignorescrouching",
        "treeHarvestWithoutCrouching" => "treeharvestwithoutcrouching",
        _ => key,
    }
}


// Config files are private to this mod and do not appear on the native Config page.
// The legacy registered settings are migrated by instalar.fish before replacement.
pub fn saved_value(host: &dyn Host, key: &str) -> Option<String> {
    let path = mod_api::ConfigPath::new(&format!("{ID}/{}", storage_key(key))).ok()?;
    let bytes = host.config_files().read(&path).ok()?;
    String::from_utf8(bytes).ok().map(|value| value.trim().to_owned())
}

pub fn save(host: &dyn Host, key: &str, value: &str) -> bool {
    let Ok(path) = mod_api::ConfigPath::new(&format!("{ID}/{}", storage_key(key))) else {
        return false;
    };
    host.config_files().write(&path, value.as_bytes()).is_ok()
}

pub fn on(host: &dyn Host, key: &str) -> bool {
    saved_value(host, key).as_deref() == Some("true")
}

pub fn flag(host: &dyn Host, key: &str, default: bool) -> bool {
    saved_value(host, key).map_or(default, |value| value == "true")
}

pub fn amount(host: &dyn Host, key: &str, default: f64, min: f64, max: f64) -> f64 {
    saved_value(host, key)
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(default)
        .clamp(min, max)
}

pub fn register(r: &mut dyn Registrar) {
    // Only the opener belongs on Sunrise's native Config page.
    // Every module and option is now stored through the private config service.
    let _ = r.register_setting(SettingSpec::choice(
        "open_ui", "qol.config.open", ["qol.config.open.value"], 0,
    ));
}
