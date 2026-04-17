use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

const SKILLSH_CACHE_DIR: &str = "marketplace/skillssh";
const SKILL_DETAIL_FILE_NAME: &str = "detail.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMarketplaceSkillDetail {
    #[serde(default)]
    pub description: String,
    pub markdown: String,
    #[serde(default)]
    pub raw_markdown: String,
    pub fetched_at_epoch_secs: u64,
}

pub fn load_skill_detail(
    cache_root_dir: &Path,
    source: &str,
    skill_id: &str,
) -> Result<Option<CachedMarketplaceSkillDetail>, String> {
    let file_path = detail_file_path(cache_root_dir, source, skill_id);
    if !file_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&file_path)
        .map_err(|error| format!("Failed to read marketplace cache file: {error}"))?;
    let cached = serde_json::from_str::<CachedMarketplaceSkillDetail>(&contents)
        .map_err(|error| format!("Failed to parse marketplace cache file: {error}"))?;

    Ok(Some(cached))
}

pub fn save_skill_detail(
    cache_root_dir: &Path,
    source: &str,
    skill_id: &str,
    description: &str,
    markdown: &str,
    raw_markdown: &str,
) -> Result<(), String> {
    let file_path = detail_file_path(cache_root_dir, source, skill_id);
    if let Some(parent_dir) = file_path.parent() {
        fs::create_dir_all(parent_dir)
            .map_err(|error| format!("Failed to create marketplace cache directory: {error}"))?;
    }

    let cached = CachedMarketplaceSkillDetail {
        description: description.to_string(),
        markdown: markdown.to_string(),
        raw_markdown: raw_markdown.to_string(),
        fetched_at_epoch_secs: current_epoch_secs()?,
    };
    let contents = serde_json::to_string_pretty(&cached)
        .map_err(|error| format!("Failed to serialize marketplace cache file: {error}"))?;

    fs::write(file_path, contents)
        .map_err(|error| format!("Failed to write marketplace cache file: {error}"))
}

pub fn is_cache_fresh(cached: &CachedMarketplaceSkillDetail, ttl_secs: u64) -> bool {
    match current_epoch_secs() {
        Ok(now) => now.saturating_sub(cached.fetched_at_epoch_secs) <= ttl_secs,
        Err(_) => false,
    }
}

fn detail_file_path(cache_root_dir: &Path, source: &str, skill_id: &str) -> PathBuf {
    cache_root_dir
        .join(SKILLSH_CACHE_DIR)
        .join(sanitize_path_component(source))
        .join(sanitize_path_component(skill_id))
        .join(SKILL_DETAIL_FILE_NAME)
}

fn sanitize_path_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => ch,
            _ => '_',
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}

fn current_epoch_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("Failed to read system time: {error}"))
}
