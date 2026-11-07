use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBundleEntry {
    pub path: String,
    pub sha1: String,
    /// VeloxClient: limits the bundle to one OS (values of `constants::TARGET_OS`).
    /// `None` means every OS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleManifest {
    pub versions: HashMap<String, HashMap<String, Vec<RemoteBundleEntry>>>,
}

impl BundleManifest {
    pub fn remote_paths(&self) -> Vec<RemoteBundleRef> {
        let mut refs = Vec::new();

        for (mc_version, loaders) in &self.versions {
            for (loader, entries) in loaders {
                for entry in entries.iter().filter(|entry| is_for_this_os(entry)) {
                    refs.push(RemoteBundleRef {
                        mc_version: mc_version.clone(),
                        loader: loader.clone(),
                        remote_path: entry.path.clone(),
                        sha1: entry.sha1.clone(),
                    });
                }
            }
        }

        refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteBundleRef {
    pub mc_version: String,
    pub loader: String,
    pub remote_path: String,
    pub sha1: String,
}

fn is_for_this_os(entry: &RemoteBundleEntry) -> bool {
    entry
        .os
        .as_deref()
        .is_none_or(|os| os == oneclient_common::constants::TARGET_OS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> BundleManifest {
        serde_json::from_str(
            r#"{"versions":{"26.3":{"fabric":[
                {"path":"/win.mrpack","sha1":"a","os":"windows"},
                {"path":"/mac.mrpack","sha1":"b","os":"osx"},
                {"path":"/all.mrpack","sha1":"c"}
            ]}}}"#,
        )
        .expect("valid manifest")
    }

    #[test]
    fn only_bundles_for_this_os_or_every_os_are_offered() {
        let paths: Vec<String> = manifest()
            .remote_paths()
            .into_iter()
            .map(|r| r.remote_path)
            .collect();

        let own = if cfg!(windows) { "/win.mrpack" } else { "/mac.mrpack" };
        let foreign = if cfg!(windows) { "/mac.mrpack" } else { "/win.mrpack" };
        assert!(paths.contains(&own.to_string()), "{own} missing from {paths:?}");
        assert!(paths.contains(&"/all.mrpack".to_string()));
        assert!(!paths.contains(&foreign.to_string()), "{foreign} must be skipped");
    }
}
