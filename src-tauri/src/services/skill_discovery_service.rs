use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::infrastructure::utils::path::normalize_path;
use crate::repositories::marketplace_install_repository::MarketplaceInstallRepository;
use crate::scanners::skill_scanner;
use crate::services::ServiceError;

#[derive(Clone)]
pub struct SkillDiscoveryService {
    install_repo: Arc<dyn MarketplaceInstallRepository>,
}

impl SkillDiscoveryService {
    pub fn new(install_repo: Arc<dyn MarketplaceInstallRepository>) -> Self {
        Self { install_repo }
    }

    pub fn list_local_skills(&self, scan_targets: Vec<SkillScanTargetDto>) -> Vec<LocalSkillSummaryDto> {
        println!(
            "[skills] list_local_skills service targets: {:?}",
            scan_targets
                .iter()
                .map(|target| format!(
                    "{}|{}|{}",
                    target.agent_id, target.agent_type, target.root_path
                ))
                .collect::<Vec<_>>()
        );

        let installs_by_path = self.marketplace_installs_by_path();

        let skills: Vec<LocalSkillSummaryDto> = skill_scanner::scan_skills(scan_targets)
            .into_iter()
            .map(|mut skill| {
                self.apply_marketplace_install_metadata_to_summary(&mut skill.summary, &installs_by_path);
                skill.summary
            })
            .collect();

        println!(
            "[skills] list_local_skills service result: {:?}",
            skills
                .iter()
                .map(|skill| format!(
                    "{}|owner={}|agentType={}|path={}",
                    skill.id, skill.owner_agent_id, skill.agent_type, skill.skill_path
                ))
                .collect::<Vec<_>>()
        );

        skills
    }

    pub fn get_local_skill_detail(
        &self,
        scan_targets: Vec<SkillScanTargetDto>,
        skill_id: &str,
    ) -> Result<LocalSkillDetailDto, ServiceError> {
        println!(
            "[skills] get_local_skill_detail service request: skill_id={}, targets={:?}",
            skill_id,
            scan_targets
                .iter()
                .map(|target| format!(
                    "{}|{}|{}",
                    target.agent_id, target.agent_type, target.root_path
                ))
                .collect::<Vec<_>>()
        );

        let installs_by_path = self.marketplace_installs_by_path();
        let skills_by_id: HashMap<_, _> = skill_scanner::scan_skills(scan_targets)
            .into_iter()
            .map(|mut skill| {
                self.apply_marketplace_install_metadata_to_detail(&mut skill.detail, &installs_by_path);
                (skill.detail.id.clone(), skill.detail)
            })
            .collect();

        let detail = skills_by_id
            .get(skill_id)
            .cloned()
            .ok_or_else(|| ServiceError::SkillNotFound(skill_id.to_string()))?;

        println!(
            "[skills] get_local_skill_detail service result: {}|owner={}|agentType={}|path={}",
            detail.id, detail.owner_agent_id, detail.agent_type, detail.skill_path
        );

        Ok(detail)
    }

    fn marketplace_installs_by_path(&self) -> HashMap<String, (String, String)> {
        self.install_repo
            .find_all()
            .into_iter()
            .map(|record| {
                (
                    normalize_path(Path::new(&record.skill_path)),
                    (record.source, record.skill_id),
                )
            })
            .collect()
    }

    fn apply_marketplace_install_metadata_to_summary(
        &self,
        summary: &mut LocalSkillSummaryDto,
        installs_by_path: &HashMap<String, (String, String)>,
    ) {
        if let Some((source, skill_id)) = installs_by_path.get(&normalize_path(Path::new(&summary.skill_path))) {
            summary.marketplace_source = Some(source.clone());
            summary.marketplace_skill_id = Some(skill_id.clone());
        }
    }

    fn apply_marketplace_install_metadata_to_detail(
        &self,
        detail: &mut LocalSkillDetailDto,
        installs_by_path: &HashMap<String, (String, String)>,
    ) {
        if let Some((source, skill_id)) = installs_by_path.get(&normalize_path(Path::new(&detail.skill_path))) {
            detail.marketplace_source = Some(source.clone());
            detail.marketplace_skill_id = Some(skill_id.clone());
        }
    }
}
