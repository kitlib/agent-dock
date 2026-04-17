use std::collections::HashMap;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::persistence::marketplace_install_store;
use crate::scanners::skill_scanner;

pub fn list_local_skills(scan_targets: Vec<SkillScanTargetDto>) -> Vec<LocalSkillSummaryDto> {
    let installs_by_path = marketplace_installs_by_path();

    skill_scanner::scan_skills(scan_targets)
        .into_iter()
        .map(|mut skill| {
            apply_marketplace_install_metadata_to_summary(&mut skill.summary, &installs_by_path);
            skill.summary
        })
        .collect()
}

pub fn get_local_skill_detail(
    scan_targets: Vec<SkillScanTargetDto>,
    skill_id: &str,
) -> Result<LocalSkillDetailDto, String> {
    let installs_by_path = marketplace_installs_by_path();
    let skills_by_id: HashMap<_, _> = skill_scanner::scan_skills(scan_targets)
        .into_iter()
        .map(|mut skill| {
            apply_marketplace_install_metadata_to_detail(&mut skill.detail, &installs_by_path);
            (skill.detail.id.clone(), skill.detail)
        })
        .collect();

    skills_by_id
        .get(skill_id)
        .cloned()
        .ok_or_else(|| format!("Skill not found: {skill_id}"))
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn marketplace_installs_by_path() -> HashMap<String, (String, String)> {
    marketplace_install_store::load_marketplace_installs()
        .into_iter()
        .map(|record| {
            (
                normalize_path(&record.skill_path),
                (record.source, record.skill_id),
            )
        })
        .collect()
}

fn apply_marketplace_install_metadata_to_summary(
    summary: &mut LocalSkillSummaryDto,
    installs_by_path: &HashMap<String, (String, String)>,
) {
    if let Some((source, skill_id)) = installs_by_path.get(&normalize_path(&summary.skill_path)) {
        summary.marketplace_source = Some(source.clone());
        summary.marketplace_skill_id = Some(skill_id.clone());
    }
}

fn apply_marketplace_install_metadata_to_detail(
    detail: &mut LocalSkillDetailDto,
    installs_by_path: &HashMap<String, (String, String)>,
) {
    if let Some((source, skill_id)) = installs_by_path.get(&normalize_path(&detail.skill_path)) {
        detail.marketplace_source = Some(source.clone());
        detail.marketplace_skill_id = Some(skill_id.clone());
    }
}
