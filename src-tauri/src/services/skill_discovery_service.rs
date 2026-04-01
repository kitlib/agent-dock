use std::collections::HashMap;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::scanners::skill_scanner;

pub fn list_local_skills(scan_targets: Vec<SkillScanTargetDto>) -> Vec<LocalSkillSummaryDto> {
    skill_scanner::scan_skills(scan_targets)
        .into_iter()
        .map(|skill| skill.summary)
        .collect()
}

pub fn get_local_skill_detail(
    scan_targets: Vec<SkillScanTargetDto>,
    skill_id: &str,
) -> Result<LocalSkillDetailDto, String> {
    let skills_by_id: HashMap<_, _> = skill_scanner::scan_skills(scan_targets)
        .into_iter()
        .map(|skill| (skill.detail.id.clone(), skill.detail))
        .collect();

    skills_by_id
        .get(skill_id)
        .cloned()
        .ok_or_else(|| format!("Skill not found: {skill_id}"))
}
