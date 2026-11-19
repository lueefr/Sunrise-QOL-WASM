//! A deliberately small in-process Sunrise runtime test for the exact release .smod.
//! Copied into the SDK-pinned Sunrise source checkout by tools/sandbox/run.sh.
#[path = "support/no_hooks.rs"]
mod no_hooks;
#[path = "../src/test_stack.rs"]
mod test_stack;

use mod_api::{Host, ServerHost};
use sunrise_mod_package::manifest::Package;
use sunrise_mod_wasm::{WasmPackage, load};
use std::path::Path;

struct Sandbox;
impl Host for Sandbox {}
mod_api::impl_host_areas!(Sandbox);
impl ServerHost for Sandbox {}
mod_api::impl_server_host_areas!(Sandbox);

#[test]
fn sunrise_qol_links_initializes_and_dispatches_wasm_hook() {
    test_stack::on_guest_stack(|| {
        let package_path = std::env::var("SUNRISE_QOL_SMOD").expect("exact .smod from build");
        let standard_path = std::env::var("SUNRISE_WASM_STD").expect("matching wasm standard");
        let path = Path::new(&package_path);
        let parsed = Package::read(path).expect("smod manifest and hashes must be valid");
        let manifest = parsed.manifest;
        assert_eq!(manifest.id, "horizon_qol_suite");
        assert!(manifest.inject.is_empty(), "sandbox requires an unstaged, API-only mod");
        let info = mod_api::PackageInfo::new(
            manifest.id.clone(),
            manifest.version.clone(),
            mod_api::PackageKind::Code,
            mod_api::Side::Both,
            vec![manifest.id.clone()],
            vec![],
        );
        let standard = std::fs::read(standard_path).expect("matching wasm standard");
        let guest = Package::read_wasm(path).expect("mod.wasm in package");
        mod_api::__private::initialize().expect("initialize host API");
        let loaded = load(&standard, vec![WasmPackage {
            manifest,
            module: guest,
        }]);
        assert!(loaded.refused.is_empty(), "Sunrise refused the mod: {:?}", loaded.refused);
        assert_eq!(loaded.mods.len(), 1, "one loadable QOL guest is required");
        loaded.initialize(&[info]).expect("execute guest registration in Wasmtime");
        let _registration =
            mod_api::__private::PhaseOwner::scope(mod_api::__private::ModPhase::Registration);
        mod_api::testing::with_mods(loaded.mods.into(), || {
            mod_api::__private::mark_game_thread();
            let _phase = mod_api::__private::PhaseOwner::scope(mod_api::__private::ModPhase::World);
            let mut sandbox = Sandbox;
            // This calls the real guest's server_stopped hook and checks that the
            // result returns through Sunrise's generated Wire transport.
            mod_api::dispatch_server_stopped(&mut sandbox, false);
        });
    });
}
