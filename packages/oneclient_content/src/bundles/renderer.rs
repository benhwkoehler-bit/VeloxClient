//! VeloxClient: switches between the standard OpenGL path and Pure Metal.
//!
//! Pure Metal (Apple Silicon, Minecraft 26.3) renders through Apple's Metal
//! and runs together with Sodium, Sodium Extra and Reese's Sodium Options.
//! What it cannot run next to is turned off while it is picked and back on
//! when the standard renderer is picked. The lists hold package ids: Modrinth
//! project ids, and `ext:cf:<id>` for files the pack takes from CurseForge.
//! `tools/build_packs.py` reads them to check that every pack mod which needs
//! a switched-off mod is listed too.

use oneclient_common::domain::GameLoader;
use oneclient_db::dao::cluster_bundle as bundle_dao;
use oneclient_db::models::OverrideType;

use crate::bundles::install::{set_artifact_enabled_to, set_bundle_package_override};
use crate::bundles::manager::BundlesManager;
use crate::ctx::ContentCtx;
use crate::error::ContentResult;
use crate::packages::store::PackageStore;

/// Only shipped (switched off, hidden) in the Apple Silicon 26.3 pack.
pub const METAL_ONLY_PROJECTS: &[&str] = &[
    "ext:cf:1732201", // Pure Metal (CurseForge)
];

/// Mods that must be off under Pure Metal: Iris (Pure Metal has no shader
/// support), Slipstream (only speeds up the OpenGL path) and mods that call
/// OpenGL directly, which aborts the JVM without a GL context.
pub const STANDARD_ONLY_PROJECTS: &[&str] = &[
    "YL57xq9U", // Iris
    "XDGYMuZ6", // Slipstream
    "xedaBnSm", // Put A Plug In It (glGetString every tick)
];

/// Whether the file belongs to one renderer, and if so whether it should be
/// on for the chosen one. `None` for every other mod.
#[must_use]
pub fn renderer_wants(package_id: &str, metal: bool) -> Option<bool> {
    if METAL_ONLY_PROJECTS.contains(&package_id) {
        Some(metal)
    } else if STANDARD_ONLY_PROJECTS.contains(&package_id) {
        Some(!metal)
    } else {
        None
    }
}

/// Always an explicit override, even when it matches the pack default: the
/// launch-time heal turns hidden bundle files (Pure Metal) back on unless a
/// `Disabled` override says the user wants them off.
#[must_use]
pub fn renderer_override(want: bool) -> OverrideType {
    if want {
        OverrideType::Enabled
    } else {
        OverrideType::Disabled
    }
}

/// Brings every renderer-bound mod of the cluster in line with `metal`: the
/// ones already installed and the ones the next bundle pass installs.
///
/// Returns whether Pure Metal is now switched on in this cluster (the pack
/// ships it and `metal` asked for it).
#[tracing::instrument(level = "debug", skip(bundles, ctx))]
pub async fn apply_renderer(
    cluster_id: i64,
    metal: bool,
    bundles: &BundlesManager,
    ctx: &ContentCtx,
) -> ContentResult<bool> {
    let cluster = PackageStore::get_cluster(cluster_id, ctx).await?;
    if !cluster.uses_bundles() {
        return Ok(false);
    }
    let Some(loader) = GameLoader::from_repr(cluster.mc_loader as u8) else {
        return Ok(false);
    };

    let archives = bundles.archives_for(ctx, &cluster.mc_version, loader).await?;
    let tracked = bundle_dao::list_bundle_tracked(&ctx.db, cluster_id).await?;
    let mut metal_on = false;

    for archive in &archives {
        let bundle_name = archive.manifest.name.as_str();
        for file in &archive.manifest.files {
            let package_id = file.kind.package_id();
            let Some(want) = renderer_wants(&package_id, metal) else {
                continue;
            };
            metal_on |= want && METAL_ONLY_PROJECTS.contains(&package_id.as_str());

            set_bundle_package_override(
                cluster_id,
                bundle_name,
                &package_id,
                Some(renderer_override(want)),
                ctx,
            )
            .await?;

            let wrong_state = tracked.iter().filter(|row| {
                row.bundle_name.as_deref() == Some(bundle_name)
                    && row.package_id.as_deref() == Some(package_id.as_str())
                    && (row.enabled != 0) != want
            });
            for row in wrong_state {
                set_artifact_enabled_to(cluster_id, &row.hash, want, ctx).await?;
            }
        }
    }

    tracing::info!(cluster_id, metal, metal_on, "renderer mods switched");
    Ok(metal_on)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PURE_METAL: &str = "ext:cf:1732201";
    const SODIUM: &str = "AANobbMI";
    const IRIS: &str = "YL57xq9U";

    #[test]
    fn pure_metal_turns_iris_off_and_itself_on() {
        assert_eq!(renderer_wants(PURE_METAL, true), Some(true));
        assert_eq!(renderer_wants(IRIS, true), Some(false));
        assert_eq!(renderer_wants("xedaBnSm", true), Some(false));
    }

    #[test]
    fn standard_renderer_keeps_iris_and_turns_pure_metal_off() {
        assert_eq!(renderer_wants(PURE_METAL, false), Some(false));
        assert_eq!(renderer_wants(IRIS, false), Some(true));
        assert_eq!(renderer_wants("XDGYMuZ6", false), Some(true));
    }

    #[test]
    fn sodium_stays_untouched_because_pure_metal_runs_with_it() {
        assert_eq!(renderer_wants(SODIUM, true), None);
        assert_eq!(renderer_wants(SODIUM, false), None);
    }

    #[test]
    fn switching_off_is_always_recorded_so_the_heal_keeps_it_off() {
        assert_eq!(renderer_override(false), OverrideType::Disabled);
        assert_eq!(renderer_override(true), OverrideType::Enabled);
    }
}
