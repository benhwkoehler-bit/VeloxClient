//! VeloxClient: Lunar Client's Fabric profiles live in `~/.lunarclient/profiles/<name>`,
//! each with one `game-versions/<minecraft version>` folder and its own `mods`.
//!
//! Only the Fabric mods the user added can come along; Lunar's built-in
//! features are part of its closed client and stay behind. Lunar's own
//! libraries in `mods/dependencies` are skipped by the importer.

use std::path::{Path, PathBuf};

use crate::LauncherResult;
use oneclient_common::domain::GameLoader;

use super::{ExternalDetection, ExternalInstance, ExternalLauncher, ExternalSettings};

const ROOT_FOLDER: &str = ".lunarclient";
const PROFILES_DIR: &str = "profiles";
const VERSIONS_DIR: &str = "game-versions";

pub fn default_root() -> Option<PathBuf> {
    let candidates = [
        super::env_dir("LUNAR_CLIENT_DIR"),
        directories::BaseDirs::new().map(|d| d.home_dir().join(ROOT_FOLDER)),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|root| looks_like_root(root))
}

pub fn looks_like_root(root: &Path) -> bool {
    root.join(PROFILES_DIR).is_dir()
}

#[tracing::instrument]
pub async fn detect_at(root: &Path) -> LauncherResult<Option<ExternalDetection>> {
    if !looks_like_root(root) {
        return Ok(None);
    }

    let mut instances = Vec::new();
    let mut entries = polyio::read_dir(root.join(PROFILES_DIR)).await?;
    while let Some(entry) = entries.next_entry().await? {
        let game_dir = entry.path();
        if !game_dir.is_dir() {
            continue;
        }
        let Some(mc_version) = single_game_version(&game_dir).await else {
            tracing::debug!(profile = %game_dir.display(), "lunar: not a Fabric profile");
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        instances.push(ExternalInstance {
            launcher: ExternalLauncher::Lunar,
            id: name.clone(),
            name,
            icon: Some(game_dir.join("icon.png")).filter(|icon| icon.is_file()),
            game_dir,
            mc_version,
            loader: GameLoader::Fabric,
            loader_version: None,
            notes: None,
            groups: Vec::new(),
            played_secs: 0,
            settings: ExternalSettings::default(),
            linked_pack: None,
            unsupported_loader: None,
            skipped_parts: Vec::new(),
        });
    }

    instances.sort_by_key(|instance| instance.name.to_lowercase());
    tracing::info!(instances = instances.len(), "detected Lunar Client");

    Ok(Some(ExternalDetection {
        launcher: ExternalLauncher::Lunar,
        root: root.to_path_buf(),
        instances,
        problem: None,
    }))
}

/// A Lunar Fabric profile has exactly one `game-versions/<version>` folder;
/// anything else (none, or several) is not something to import
async fn single_game_version(profile: &Path) -> Option<String> {
    let mut entries = polyio::read_dir(profile.join(VERSIONS_DIR)).await.ok()?;
    let mut versions = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().is_dir() {
            versions.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    match versions.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn profiles_with_one_game_version_are_offered() {
        let dir = polyio::tempdir().await.unwrap();
        let root = dir.dir_path();
        let profiles = root.join(PROFILES_DIR);
        polyio::create_dir_all(profiles.join("haupt").join(VERSIONS_DIR).join("26.3"))
            .await
            .unwrap();
        polyio::create_dir_all(profiles.join("haupt").join("mods")).await.unwrap();
        // no game version: Lunar's own bookkeeping, not a profile
        polyio::create_dir_all(profiles.join("meine-mods").join("mods"))
            .await
            .unwrap();

        let detection = detect_at(root).await.unwrap().unwrap();

        assert_eq!(detection.instances.len(), 1);
        let profile = &detection.instances[0];
        assert_eq!(profile.name, "haupt");
        assert_eq!(profile.mc_version, "26.3");
        assert_eq!(profile.loader, GameLoader::Fabric);
        assert_eq!(profile.game_dir, profiles.join("haupt"));
    }

    #[tokio::test]
    async fn a_folder_without_profiles_is_not_lunar() {
        let dir = polyio::tempdir().await.unwrap();
        assert!(detect_at(dir.dir_path()).await.unwrap().is_none());
    }
}
