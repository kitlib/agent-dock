use crate::repositories::errors::RepositoryError;

/// Marketplace install record
#[derive(Clone, Debug)]
pub struct MarketplaceInstallRecord {
    pub source: String,
    pub skill_id: String,
    pub install_method: String,
    pub skill_path: String,
    pub entry_file_path: String,
    pub installed_at: String,
}

/// Repository for marketplace installs
pub trait MarketplaceInstallRepository: Send + Sync {
    fn find_all(&self) -> Vec<MarketplaceInstallRecord>;
    fn save_all(&self, records: &[MarketplaceInstallRecord]) -> Result<(), RepositoryError>;
    fn upsert(&self, record: MarketplaceInstallRecord) -> Result<(), RepositoryError>;
    fn delete(&self, skill_path: &str, entry_file_path: &str) -> Result<(), RepositoryError>;
}
