//! VeloxClient: the wardrobe's skin library, with or without Poly+.
//!
//! With Poly+ on, skins are saved to the Poly+ account as before. With Poly+
//! off (its terms declined), the library lives in a JSON file on this
//! computer instead, so uploading and switching skins works for everyone.
//! Only the texture hash is stored: Mojang hosts every uploaded skin under
//! textures.minecraft.net, so wearing one again is just a link.

use std::path::{Path, PathBuf};

use oneclient_auth::MinecraftAccount;
use oneclient_polyplus::PlusError;
use oneclient_polyplus::api::{self, SkinInfo, SkinList, SkinModel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How many skins the local library keeps per account
pub const LOCAL_SKIN_LIMIT: u64 = 60;
const TEXTURE_BASE: &str = "https://textures.minecraft.net/texture/";
const MAX_NAME_CHARS: usize = 64;

/// Serialises file access so two quick actions never overwrite each other
static FILE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn list_skins(account: &MinecraftAccount) -> Result<SkinList, PlusError> {
    if !use_local() {
        return api::list_skins(account).await;
    }
    let _guard = FILE_LOCK.lock().await;
    Ok(load(&library_path(account.id)?)?.to_list())
}

pub async fn add_skin(
    account: &MinecraftAccount,
    texture_hash: &str,
    model: SkinModel,
    name: Option<&str>,
) -> Result<SkinInfo, PlusError> {
    if !use_local() {
        return api::add_skin(account, texture_hash, model, name).await;
    }
    let path = library_path(account.id)?;
    let _guard = FILE_LOCK.lock().await;
    let (library, info) = load(&path)?.with_skin(texture_hash, model, name, &now())?;
    save(&path, &library)?;
    Ok(info)
}

pub async fn update_skin(
    account: &MinecraftAccount,
    id: i32,
    changes: &serde_json::Value,
) -> Result<SkinInfo, PlusError> {
    if !use_local() {
        return api::update_skin(account, id, changes).await;
    }
    let path = library_path(account.id)?;
    let _guard = FILE_LOCK.lock().await;
    let (library, info) = load(&path)?.with_changes(id, changes)?;
    save(&path, &library)?;
    Ok(info)
}

pub async fn delete_skin(account: &MinecraftAccount, id: i32) -> Result<(), PlusError> {
    if !use_local() {
        return api::delete_skin(account, id).await;
    }
    let path = library_path(account.id)?;
    let _guard = FILE_LOCK.lock().await;
    let library = load(&path)?.without(id)?;
    save(&path, &library)
}

/// True while Poly+ is off, so the library lives on this computer
pub fn use_local() -> bool {
    oneclient_polyplus::plus_blocked()
}

fn library_path(account: Uuid) -> Result<PathBuf, PlusError> {
    let data = oneclient_common::paths::data_dir().map_err(std::io::Error::other)?;
    Ok(data.join("skins").join(format!("{account}.json")))
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Library {
    next_id: i32,
    skins: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    id: i32,
    name: String,
    slim: bool,
    hash: String,
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    created_at: String,
}

impl Entry {
    fn info(&self) -> SkinInfo {
        SkinInfo {
            id: self.id,
            name: self.name.clone(),
            model: if self.slim {
                SkinModel::Slim
            } else {
                SkinModel::Classic
            },
            url: format!("{TEXTURE_BASE}{}", self.hash),
            hash: self.hash.clone(),
            favorite: self.favorite,
            created_at: self.created_at.clone(),
        }
    }
}

fn not_found() -> PlusError {
    PlusError::Api {
        status: 404,
        body: String::new(),
    }
}

fn clean_name(name: &str) -> String {
    name.trim().chars().take(MAX_NAME_CHARS).collect()
}

impl Library {
    fn to_list(&self) -> SkinList {
        SkinList {
            skins: self.skins.iter().rev().map(Entry::info).collect(),
            limit: LOCAL_SKIN_LIMIT,
        }
    }

    /// Adds a skin, or returns the saved one when the texture is already there
    fn with_skin(
        &self,
        hash: &str,
        model: SkinModel,
        name: Option<&str>,
        created_at: &str,
    ) -> Result<(Self, SkinInfo), PlusError> {
        let hash = hash.to_ascii_lowercase();
        if let Some(existing) = self.skins.iter().find(|skin| skin.hash == hash) {
            return Ok((self.clone(), existing.info()));
        }
        if self.skins.len() as u64 >= LOCAL_SKIN_LIMIT {
            return Err(PlusError::Api {
                status: 409,
                body: String::new(),
            });
        }
        let id = self.next_id.max(0) + 1;
        let name = name
            .map(clean_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| format!("Skin {id}"));
        let entry = Entry {
            id,
            name,
            slim: model == SkinModel::Slim,
            hash,
            favorite: false,
            created_at: created_at.to_string(),
        };
        let info = entry.info();
        let skins = self.skins.iter().cloned().chain([entry]).collect();
        Ok((Self { next_id: id, skins }, info))
    }

    /// Applies the same `name` / `favorite` / `model` changes Poly+ accepts
    fn with_changes(
        &self,
        id: i32,
        changes: &serde_json::Value,
    ) -> Result<(Self, SkinInfo), PlusError> {
        let original = self
            .skins
            .iter()
            .find(|skin| skin.id == id)
            .ok_or_else(not_found)?;
        let changed = changed_entry(original, changes);
        let info = changed.info();
        let skins = self
            .skins
            .iter()
            .map(|skin| {
                if skin.id == id {
                    changed.clone()
                } else {
                    skin.clone()
                }
            })
            .collect();
        Ok((
            Self {
                next_id: self.next_id,
                skins,
            },
            info,
        ))
    }

    fn without(&self, id: i32) -> Result<Self, PlusError> {
        if !self.skins.iter().any(|skin| skin.id == id) {
            return Err(not_found());
        }
        Ok(Self {
            next_id: self.next_id,
            skins: self
                .skins
                .iter()
                .filter(|skin| skin.id != id)
                .cloned()
                .collect(),
        })
    }
}

fn changed_entry(original: &Entry, changes: &serde_json::Value) -> Entry {
    let name = changes
        .get("name")
        .and_then(|v| v.as_str())
        .map(clean_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| original.name.clone());
    let favorite = changes
        .get("favorite")
        .and_then(|v| v.as_bool())
        .unwrap_or(original.favorite);
    let slim = changes
        .get("model")
        .and_then(|v| v.as_str())
        .map_or(original.slim, |model| model == "slim");
    Entry {
        name,
        favorite,
        slim,
        ..original.clone()
    }
}

fn load(path: &Path) -> Result<Library, PlusError> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|err| {
            tracing::warn!(
                "[wardrobe] local skin library {} is unreadable: {err}",
                path.display()
            );
            PlusError::Io(std::io::Error::other(err))
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Library::default()),
        Err(err) => Err(err.into()),
    }
}

/// Written to a temp file first so a crash never leaves half a library
fn save(path: &Path, library: &Library) -> Result<(), PlusError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(library).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn added(library: &Library, hash: &str, name: Option<&str>) -> (Library, SkinInfo) {
        library
            .with_skin(hash, SkinModel::Classic, name, "2026-11-07T00:00:00Z")
            .expect("adding works")
    }

    #[test]
    fn adding_a_skin_links_to_the_mojang_texture() {
        let (library, info) = added(&Library::default(), HASH_A, Some("  Mein Skin "));
        assert_eq!(info.id, 1);
        assert_eq!(info.name, "Mein Skin");
        assert_eq!(info.url, format!("{TEXTURE_BASE}{HASH_A}"));
        assert_eq!(api::texture_hash(&info.url).as_deref(), Some(HASH_A));
        assert_eq!(library.to_list().skins.len(), 1);
    }

    #[test]
    fn the_same_texture_is_only_saved_once() {
        let (library, first) = added(&Library::default(), HASH_A, None);
        let (library, again) = added(&library, &HASH_A.to_uppercase(), Some("other"));
        assert_eq!(first, again);
        assert_eq!(library.skins.len(), 1);
    }

    #[test]
    fn unnamed_skins_get_a_numbered_name_and_newest_comes_first() {
        let (library, _) = added(&Library::default(), HASH_A, None);
        let (library, second) = added(&library, HASH_B, Some("   "));
        assert_eq!(second.name, "Skin 2");
        let names: Vec<_> = library
            .to_list()
            .skins
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["Skin 2", "Skin 1"]);
    }

    #[test]
    fn changes_rename_favourite_and_switch_arms() {
        let (library, info) = added(&Library::default(), HASH_A, None);
        let changes = serde_json::json!({ "name": "Neu", "favorite": true, "model": "slim" });
        let (changed, updated) = library.with_changes(info.id, &changes).expect("found");
        assert_eq!(updated.name, "Neu");
        assert!(updated.favorite);
        assert_eq!(updated.model, SkinModel::Slim);
        assert_eq!(
            library.skins[0].name, "Skin 1",
            "the original stays untouched"
        );
        assert_eq!(changed.skins[0].name, "Neu");
    }

    #[test]
    fn unknown_ids_report_not_found() {
        let library = Library::default();
        assert!(matches!(
            library.with_changes(7, &serde_json::json!({})),
            Err(PlusError::Api { status: 404, .. })
        ));
        assert!(matches!(
            library.without(7),
            Err(PlusError::Api { status: 404, .. })
        ));
    }

    #[test]
    fn deleting_keeps_ids_unique() {
        let (library, first) = added(&Library::default(), HASH_A, None);
        let library = library.without(first.id).expect("deleted");
        let (_, next) = added(&library, HASH_B, None);
        assert_eq!(next.id, 2);
    }

    #[test]
    fn a_full_library_refuses_more() {
        let skins = (0..LOCAL_SKIN_LIMIT as i32)
            .map(|id| Entry {
                id,
                name: format!("s{id}"),
                slim: false,
                hash: format!("{id:064x}"),
                favorite: false,
                created_at: String::new(),
            })
            .collect();
        let library = Library {
            next_id: LOCAL_SKIN_LIMIT as i32,
            skins,
        };
        assert!(matches!(
            library.with_skin(HASH_A, SkinModel::Slim, None, ""),
            Err(PlusError::Api { status: 409, .. })
        ));
    }

    #[test]
    fn saving_and_loading_round_trips() {
        let dir = std::env::temp_dir().join(format!("velox-skins-{}", Uuid::new_v4()));
        let path = dir.join("account.json");
        assert_eq!(
            load(&path).expect("missing file is empty"),
            Library::default()
        );
        let (library, _) = added(&Library::default(), HASH_A, Some("Rund"));
        save(&path, &library).expect("saved");
        assert_eq!(load(&path).expect("loaded"), library);
        std::fs::remove_dir_all(dir).ok();
    }
}
