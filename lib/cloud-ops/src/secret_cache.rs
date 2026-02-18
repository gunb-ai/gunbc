//! Secret fetch planning and TTL cache helpers.

use crate::project_spec::{ProjectSpec, SecretStatus};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretCacheEntry {
    pub value: String,
    pub fetched_at_epoch_secs: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretValueCache {
    pub entries: BTreeMap<String, SecretCacheEntry>,
}

impl SecretValueCache {
    #[allow(clippy::disallowed_methods)] // Secret cache manages a local TTL cache directory for credentials.
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read secret cache '{}': {}", path.display(), e))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("failed to parse secret cache '{}': {}", path.display(), e))
    }

    #[allow(clippy::disallowed_methods)] // Secret cache manages a local TTL cache directory for credentials.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create secret cache directory '{}': {}",
                    parent.display(),
                    e
                )
            })?;
        }
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize secret cache: {e}"))?;
        std::fs::write(path, raw)
            .map_err(|e| format!("failed to write secret cache '{}': {}", path.display(), e))
    }

    pub fn upsert(
        &mut self,
        secret_id: impl Into<String>,
        value: impl Into<String>,
        now: SystemTime,
    ) {
        self.entries.insert(
            secret_id.into(),
            SecretCacheEntry {
                value: value.into(),
                fetched_at_epoch_secs: epoch_secs(now),
            },
        );
    }

    pub fn get_fresh(&self, secret_id: &str, now: SystemTime, ttl: Duration) -> Option<&str> {
        let entry = self.entries.get(secret_id)?;
        let age = epoch_secs(now).saturating_sub(entry.fetched_at_epoch_secs);
        if age <= ttl.as_secs() {
            Some(entry.value.as_str())
        } else {
            None
        }
    }
}

/// Compute which secrets need remote fetch.
///
/// Only fetches for env vars that are currently missing from the shell and
/// cache entries that are absent/stale for those missing vars.
pub fn plan_secret_fetch(
    project_spec: &ProjectSpec,
    namespace: &str,
    present_env_vars: &HashSet<String>,
    cache: &SecretValueCache,
    now: SystemTime,
    ttl: Duration,
) -> Result<Vec<String>, String> {
    let ns = project_spec
        .namespace(namespace)
        .ok_or_else(|| format!("unknown namespace '{}'", namespace))?;

    let mut to_fetch = Vec::new();
    for secret in project_spec
        .secrets
        .iter()
        .filter(|s| s.status == SecretStatus::Active)
    {
        if present_env_vars.contains(secret.env_name) {
            continue;
        }
        let prefixed = format!("{}{}", ns.secret_prefix(), secret.secret_id);
        if cache.get_fresh(&prefixed, now, ttl).is_none() {
            to_fetch.push(prefixed);
        }
    }

    Ok(to_fetch)
}

fn epoch_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_spec::GUNBAI_SECRETS;

    #[test]
    fn plan_secret_fetch_skips_present_env_and_uses_cache_ttl() {
        let now = UNIX_EPOCH + Duration::from_secs(1000);
        let mut present = HashSet::new();
        present.insert("UNRELATED".to_string());

        let mut cache = SecretValueCache::default();
        cache.upsert(
            "dev-github-token",
            "cached-token",
            now - Duration::from_secs(30),
        );

        let to_fetch = plan_secret_fetch(
            &GUNBAI_SECRETS,
            "dev",
            &present,
            &cache,
            now,
            Duration::from_secs(300),
        )
        .expect("plan should resolve");
        assert!(
            !to_fetch.contains(&"dev-github-token".to_string()),
            "fresh cache entry should avoid fetch"
        );
    }

    #[test]
    fn plan_secret_fetch_includes_stale_cache_entries() {
        let now = UNIX_EPOCH + Duration::from_secs(10_000);
        let present = HashSet::new();
        let mut cache = SecretValueCache::default();
        cache.upsert(
            "dev-github-token",
            "cached-token",
            now - Duration::from_secs(3600),
        );

        let to_fetch = plan_secret_fetch(
            &GUNBAI_SECRETS,
            "dev",
            &present,
            &cache,
            now,
            Duration::from_secs(60),
        )
        .expect("plan should resolve");
        assert_eq!(to_fetch, vec!["dev-github-token".to_string()]);
    }

    #[test]
    #[allow(clippy::disallowed_methods)] // Test-only filesystem operations for cache/config fixtures.
    fn cache_round_trip_save_and_load() {
        let now = UNIX_EPOCH + Duration::from_secs(100);
        let mut cache = SecretValueCache::default();
        cache.upsert("dev-github-token", "tok", now);

        let temp_dir =
            std::env::temp_dir().join(format!("gunbc-secret-cache-{}", std::process::id()));
        let path = temp_dir.join("cache.json");
        cache.save(&path).expect("save");
        let loaded = SecretValueCache::load(&path).expect("load");
        assert_eq!(
            loaded
                .entries
                .get("dev-github-token")
                .map(|e| e.value.as_str()),
            Some("tok")
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
