//! VeloxClient: the NoRisk Client launcher (v3) keeps its profiles in
//! `meta/app.db` (SQLite) and each profile's game directory under `profiles/`.
//!
//! Only profiles the user made are offered. NoRisk's own standard profiles
//! ("NRC 26.3", server events) run NoRisk's closed client pack and are skipped.
//! Mods NoRisk manages in its database (outside the profile's `mods` folder)
//! are not carried over; everything inside the profile folder is.

use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::LauncherResult;
use oneclient_common::domain::GameLoader;

use super::{ExternalDetection, ExternalInstance, ExternalLauncher, ExternalSettings};

const APP_FOLDER: &str = "gg.norisk.NoRiskClientV3";
const DB_FILE: &str = "app.db";
const META_DIR: &str = "meta";
const PROFILES_DIR: &str = "profiles";

pub fn default_root() -> Option<PathBuf> {
    let candidates = [
        super::env_dir("NORISK_LAUNCHER_DIR"),
        directories::BaseDirs::new().map(|d| d.data_dir().join(APP_FOLDER)),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|root| looks_like_root(root))
}

pub fn looks_like_root(root: &Path) -> bool {
    root.join(META_DIR).join(DB_FILE).is_file()
}

#[tracing::instrument]
pub async fn detect_at(root: &Path) -> LauncherResult<Option<ExternalDetection>> {
    if !looks_like_root(root) {
        return Ok(None);
    }

    // NoRisk may be running with the database in WAL mode: read a snapshot
    let snapshot = polyio::tempdir().await?;
    for suffix in ["", "-wal", "-shm"] {
        let name = format!("{DB_FILE}{suffix}");
        let src = root.join(META_DIR).join(&name);
        if src.is_file() {
            polyio::copy(&src, snapshot.dir_path().join(&name)).await?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(snapshot.dir_path().join(DB_FILE)))
        .await?;
    let result = read_profiles(root, &pool).await;
    pool.close().await;

    let mut instances = result?;
    instances.sort_by_key(|instance| instance.name.to_lowercase());
    tracing::info!(instances = instances.len(), "detected NoRisk launcher");

    Ok(Some(ExternalDetection {
        launcher: ExternalLauncher::NoRisk,
        root: root.to_path_buf(),
        instances,
        problem: None,
    }))
}

async fn read_profiles(root: &Path, pool: &SqlitePool) -> LauncherResult<Vec<ExternalInstance>> {
    let rows = sqlx::query(
        "SELECT name, path, game_version, loader, loader_version, group_name, playtime_seconds \
         FROM profiles WHERE is_standard_version = 0",
    )
    .fetch_all(pool)
    .await?;

    let profiles_dir = root.join(PROFILES_DIR);
    let mut instances = Vec::with_capacity(rows.len());
    for row in rows {
        let path: String = row.try_get("path")?;
        let game_dir = profiles_dir.join(&path);
        if !game_dir.is_dir() {
            tracing::debug!(path, "norisk: profile folder is missing");
            continue;
        }
        let Some(loader) = parse_loader(&row.try_get::<String, _>("loader")?) else {
            continue;
        };
        let played: i64 = row.try_get("playtime_seconds")?;

        instances.push(ExternalInstance {
            launcher: ExternalLauncher::NoRisk,
            id: path,
            name: row.try_get("name")?,
            game_dir,
            mc_version: row.try_get("game_version")?,
            loader,
            loader_version: row
                .try_get::<Option<String>, _>("loader_version")?
                .as_deref()
                .and_then(clean_loader_version),
            icon: None,
            notes: None,
            groups: row
                .try_get::<Option<String>, _>("group_name")?
                .filter(|group| !group.trim().is_empty())
                .into_iter()
                .collect(),
            played_secs: u64::try_from(played).unwrap_or(0),
            settings: ExternalSettings::default(),
            linked_pack: None,
            unsupported_loader: None,
            skipped_parts: Vec::new(),
        });
    }
    Ok(instances)
}

fn parse_loader(raw: &str) -> Option<GameLoader> {
    match raw.to_ascii_lowercase().as_str() {
        "vanilla" | "" => Some(GameLoader::Vanilla),
        "fabric" => Some(GameLoader::Fabric),
        "quilt" => Some(GameLoader::Quilt),
        "forge" => Some(GameLoader::Forge),
        "neoforge" => Some(GameLoader::NeoForge),
        other => {
            tracing::debug!(loader = other, "norisk: skipping profile with unknown loader");
            None
        }
    }
}

/// NoRisk writes versions like `0.19.5 (stable)`; only the number is a version
fn clean_loader_version(raw: &str) -> Option<String> {
    raw.split_whitespace()
        .next()
        .filter(|version| !version.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_version_drops_the_channel_suffix() {
        assert_eq!(clean_loader_version("0.19.5 (stable)").as_deref(), Some("0.19.5"));
        assert_eq!(clean_loader_version("0.16.14").as_deref(), Some("0.16.14"));
        assert_eq!(clean_loader_version("  "), None);
    }

    #[test]
    fn loaders_are_read_case_insensitively() {
        assert_eq!(parse_loader("Fabric"), Some(GameLoader::Fabric));
        assert_eq!(parse_loader("forge"), Some(GameLoader::Forge));
        assert_eq!(parse_loader("liteloader"), None);
    }

    #[tokio::test]
    async fn only_user_profiles_with_a_folder_are_offered() {
        let dir = polyio::tempdir().await.unwrap();
        let root = dir.dir_path();
        polyio::create_dir_all(root.join(META_DIR)).await.unwrap();
        polyio::create_dir_all(root.join(PROFILES_DIR).join("Fabric 26.3").join("mods"))
            .await
            .unwrap();

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(root.join(META_DIR).join(DB_FILE))
                    .create_if_missing(true),
            )
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE profiles (id TEXT, name TEXT, path TEXT, game_version TEXT, \
             loader TEXT, loader_version TEXT, group_name TEXT, \
             playtime_seconds INTEGER, is_standard_version INTEGER)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO profiles VALUES \
             ('a', 'Hauptmods', 'Fabric 26.3', '26.3', 'fabric', '0.19.5 (stable)', NULL, 120, 0), \
             ('b', 'NRC 26.3', 'noriskclient/new', '26.3', 'fabric', '0.19.5', 'NORISK CLIENT', 0, 1), \
             ('c', 'Gone', 'Missing', '26.2', 'fabric', NULL, NULL, 0, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        let detection = detect_at(root).await.unwrap().unwrap();

        assert_eq!(detection.instances.len(), 1);
        let profile = &detection.instances[0];
        assert_eq!(profile.name, "Hauptmods");
        assert_eq!(profile.mc_version, "26.3");
        assert_eq!(profile.loader, GameLoader::Fabric);
        assert_eq!(profile.loader_version.as_deref(), Some("0.19.5"));
        assert_eq!(profile.played_secs, 120);
        assert_eq!(profile.game_dir, root.join(PROFILES_DIR).join("Fabric 26.3"));
    }
}
