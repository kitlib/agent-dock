use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::infrastructure::utils::path::normalize_path;
use crate::repositories::marketplace_install_repository::{MarketplaceInstallRecord as DomainRecord, MarketplaceInstallRepository};
use crate::repositories::RepositoryError;
use serde::{Deserialize, Serialize};

const STORE_DIR_NAME: &str = ".agentdock";
const STORE_FILE_NAME: &str = "marketplace-installs.json";

#[derive(Clone, Serialize, Deserialize)]
struct JsonRecord {
    pub source: String,
    pub skill_id: String,
    pub install_method: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub installed_at: String,
}

impl From<DomainRecord> for JsonRecord {
    fn from(record: DomainRecord) -> Self {
        Self {
            source: record.source,
            skill_id: record.skill_id,
            install_method: record.install_method,
            skill_path: record.skill_path,
            entry_file_path: record.entry_file_path,
            installed_at: record.installed_at,
        }
    }
}

impl From<JsonRecord> for DomainRecord {
    fn from(record: JsonRecord) -> Self {
        Self {
            source: record.source,
            skill_id: record.skill_id,
            install_method: record.install_method,
            skill_path: record.skill_path,
            entry_file_path: record.entry_file_path,
            installed_at: record.installed_at,
        }
    }
}

pub struct JsonMarketplaceInstallRepository {
    store_path: PathBuf,
}

impl JsonMarketplaceInstallRepository {
    pub fn new() -> Self {
        let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let store_path = base_dir.join(STORE_DIR_NAME).join(STORE_FILE_NAME);
        Self { store_path }
    }

    fn ensure_store_seeded(&self) -> Vec<JsonRecord> {
        if self.store_path.exists() {
            let contents = fs::read_to_string(&self.store_path).unwrap_or_default();
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}

impl Default for JsonMarketplaceInstallRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketplaceInstallRepository for JsonMarketplaceInstallRepository {
    fn find_all(&self) -> Vec<DomainRecord> {
        self.ensure_store_seeded().into_iter().map(Into::into).collect()
    }

    fn save_all(&self, records: &[DomainRecord]) -> Result<(), RepositoryError> {
        let json_records: Vec<JsonRecord> = records.iter().cloned().map(Into::into).collect();
        let dir_path = self.store_path.parent().ok_or_else(|| RepositoryError::StorageError("Invalid store path".into()))?;
        fs::create_dir_all(dir_path).map_err(RepositoryError::IoError)?;

        let contents = serde_json::to_string_pretty(&json_records).map_err(|e| RepositoryError::SerializationError(e.to_string()))?;
        fs::write(&self.store_path, contents).map_err(RepositoryError::IoError)
    }

    fn upsert(&self, record: DomainRecord) -> Result<(), RepositoryError> {
        let mut records = self.find_all();
        let normalized_skill_path = normalize_path(Path::new(&record.skill_path));
        let normalized_entry_file_path = normalize_path(Path::new(&record.entry_file_path));

        records.retain(|existing| {
            normalize_path(Path::new(&existing.skill_path)) != normalized_skill_path
                || normalize_path(Path::new(&existing.entry_file_path)) != normalized_entry_file_path
        });
        records.push(record);

        self.save_all(&records)
    }

    fn delete(&self, skill_path: &str, entry_file_path: &str) -> Result<(), RepositoryError> {
        let normalized_skill_path = normalize_path(Path::new(skill_path));
        let normalized_entry_file_path = normalize_path(Path::new(entry_file_path));
        let mut records = self.find_all();
        records.retain(|record| {
            normalize_path(Path::new(&record.skill_path)) != normalized_skill_path
                || normalize_path(Path::new(&record.entry_file_path)) != normalized_entry_file_path
        });
        self.save_all(&records)
    }
}
