use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

const STORE_DIR_NAME: &str = ".agentdock";
const STORE_FILE_NAME: &str = "marketplace-installs.json";

#[derive(Clone, Serialize, Deserialize)]
pub struct MarketplaceInstallRecord {
    pub source: String,
    pub skill_id: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub installed_at: String,
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn store_dir_path() -> PathBuf {
    let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_dir.join(STORE_DIR_NAME)
}

fn store_file_path() -> PathBuf {
    store_dir_path().join(STORE_FILE_NAME)
}

fn ensure_store_seeded() -> Result<Vec<MarketplaceInstallRecord>, String> {
    let file_path = store_file_path();
    if file_path.exists() {
        let contents = fs::read_to_string(&file_path).map_err(|error| error.to_string())?;
        let records = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
        return Ok(records);
    }

    save_marketplace_installs(&[])?;
    Ok(Vec::new())
}

pub fn load_marketplace_installs() -> Vec<MarketplaceInstallRecord> {
    ensure_store_seeded().unwrap_or_default()
}

pub fn save_marketplace_installs(records: &[MarketplaceInstallRecord]) -> Result<(), String> {
    let dir_path = store_dir_path();
    fs::create_dir_all(&dir_path).map_err(|error| error.to_string())?;

    let file_path = dir_path.join(STORE_FILE_NAME);
    let contents = serde_json::to_string_pretty(records).map_err(|error| error.to_string())?;
    fs::write(file_path, contents).map_err(|error| error.to_string())
}

pub fn upsert_marketplace_install(record: MarketplaceInstallRecord) -> Result<(), String> {
    let mut records = load_marketplace_installs();
    let normalized_skill_path = normalize_path(&record.skill_path);
    let normalized_entry_file_path = normalize_path(&record.entry_file_path);

    records.retain(|existing| {
        normalize_path(&existing.skill_path) != normalized_skill_path
            && normalize_path(&existing.entry_file_path) != normalized_entry_file_path
    });
    records.push(record);

    save_marketplace_installs(&records)
}

pub fn remove_marketplace_install(skill_path: &str, entry_file_path: &str) -> Result<(), String> {
    let normalized_skill_path = normalize_path(skill_path);
    let normalized_entry_file_path = normalize_path(entry_file_path);
    let mut records = load_marketplace_installs();
    records.retain(|record| {
        normalize_path(&record.skill_path) != normalized_skill_path
            && normalize_path(&record.entry_file_path) != normalized_entry_file_path
    });
    save_marketplace_installs(&records)
}
