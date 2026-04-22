use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::dto::marketplace::{MarketplaceItemDto, MarketplaceSkillDetailDto, SkillsShSkillDto};
use crate::scanners::skillssh_scanner::{SkillsShSkillDetailRecord, SkillsShSkillRecord};

const SKILL_ENTRY_FILE: &str = "SKILL.md";

#[derive(Clone)]
pub struct MarketplaceService {}

impl Default for MarketplaceService {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketplaceService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn to_skillssh_skill_dto(&self, skill: SkillsShSkillRecord) -> SkillsShSkillDto {
        SkillsShSkillDto {
            id: skill.id,
            skill_id: skill.skill_id,
            name: skill.name,
            source: skill.source,
            installs: skill.installs,
        }
    }

    pub fn to_marketplace_item(&self, skill: SkillsShSkillDto) -> MarketplaceItemDto {
        let author = skill
            .source
            .split('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown")
            .to_string();
        let mut highlights = vec!["skills.sh".to_string(), format!("GitHub {}", skill.source)];
        if skill.installs > 0 {
            highlights.push(format!("{} installs", skill.installs));
        }

        MarketplaceItemDto {
            id: format!("skillssh:{}", skill.id),
            kind: "skill".into(),
            name: skill.name.clone(),
            skill_id: Some(skill.skill_id.clone()),
            author,
            source: skill.source.clone(),
            version: "latest".into(),
            installs: skill.installs,
            updated_at: Utc::now().format("%Y-%m-%d").to_string(),
            install_state: "install".into(),
            description: format!(
                "Published on skills.sh and installed from GitHub source {}.",
                skill.source
            ),
            highlights,
            url: Some(format!(
                "https://skills.sh/{}/{}",
                skill.source, skill.skill_id
            )),
        }
    }

    pub fn to_marketplace_skill_detail(&self, detail: SkillsShSkillDetailRecord) -> MarketplaceSkillDetailDto {
        MarketplaceSkillDetailDto {
            description: detail.description,
            markdown: detail.markdown,
            raw_markdown: detail.raw_markdown,
        }
    }

    pub fn marketplace_skill_directory_name(&self, skill_id: &str) -> String {
        let sanitized = skill_id
            .chars()
            .map(|ch| match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
                _ => '-',
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string();

        if sanitized.is_empty() {
            "marketplace-skill".into()
        } else {
            sanitized
        }
    }

    pub fn marketplace_skill_paths(&self, skills_root: &Path, skill_id: &str) -> (PathBuf, PathBuf) {
        let skill_path = skills_root.join(self.marketplace_skill_directory_name(skill_id));
        let entry_file_path = skill_path.join(SKILL_ENTRY_FILE);
        (skill_path, entry_file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn marketplace_skill_directory_name_sanitizes_invalid_characters() {
        let service = MarketplaceService::new();
        assert_eq!(
            service.marketplace_skill_directory_name("foo/bar baz"),
            "foo-bar-baz"
        );
    }

    #[test]
    fn marketplace_skill_paths_use_skill_md_entry() {
        let service = MarketplaceService::new();
        let (skill_path, entry_path) = service.marketplace_skill_paths(Path::new("/tmp/skills"), "demo");
        assert_eq!(skill_path, Path::new("/tmp/skills/demo"));
        assert_eq!(entry_path, Path::new("/tmp/skills/demo/SKILL.md"));
    }
}
