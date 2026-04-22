use std::str::FromStr;
use std::{collections::HashSet, path::Path, time::Duration};

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, REFERER};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::infrastructure::persistence::marketplace_cache_store;

const SKILL_DETAIL_CACHE_TTL_SECS: u64 = 60 * 60 * 24;
const MARKETPLACE_PAGE_SIZE: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkillRecord {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub source: String,
    pub installs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkillListRecord {
    pub items: Vec<SkillsShSkillRecord>,
    pub total_skills: Option<u64>,
    pub has_more: bool,
    pub page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsShSkillDetailRecord {
    pub description: String,
    pub markdown: String,
    pub raw_markdown: String,
}

#[derive(Debug, Clone)]
pub struct SkillsShSkillFileRecord {
    pub relative_path: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SkillsShSkillBundleRecord {
    pub files: Vec<SkillsShSkillFileRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkillsShDownloadFileRecord {
    path: String,
    contents: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkillsShDownloadResponse {
    files: Vec<SkillsShDownloadFileRecord>,
    #[allow(dead_code)]
    hash: String,
}

#[derive(Debug, Clone, Copy)]
pub enum LeaderboardType {
    AllTime,
    Trending,
    Hot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketplaceInstallMethod {
    SkillsSh,
    GitHub,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoInfo {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeResponse {
    tree: Vec<GitHubTreeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubTreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

#[derive(Debug, Clone)]
struct ResolvedSkillLocation {
    branch: String,
    skill_dir_path: String,
    tree: Vec<GitHubTreeEntry>,
}

impl FromStr for LeaderboardType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "trending" => Ok(Self::Trending),
            "hot" => Ok(Self::Hot),
            _ => Ok(Self::AllTime),
        }
    }
}

impl LeaderboardType {
    fn api_path(self) -> &'static str {
        match self {
            Self::AllTime => "all-time",
            Self::Trending => "trending",
            Self::Hot => "hot",
        }
    }
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
        )
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("Failed to build skills.sh client: {error}"))
}

fn build_skillssh_headers() -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(REFERER, HeaderValue::from_static("https://skills.sh/"));
    Ok(headers)
}

pub fn fetch_leaderboard(
    board: LeaderboardType,
    page: usize,
) -> Result<SkillsShSkillListRecord, String> {
    let client = build_http_client()?;
    let url = format!("https://skills.sh/api/skills/{}/{}", board.api_path(), page);
    let headers = build_skillssh_headers()?;
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .map_err(|error| format!("Failed to fetch skills.sh leaderboard: {error}"))?
        .json::<Value>()
        .map_err(|error| format!("Failed to parse skills.sh leaderboard response: {error}"))?;

    parse_skill_list_response(&response, page)
}

pub fn search_skills(
    query: &str,
    page_size: usize,
    page: usize,
) -> Result<SkillsShSkillListRecord, String> {
    let client = build_http_client()?;
    let bounded_page_size = page_size.clamp(1, 100);
    let bounded_limit = ((page + 1) * bounded_page_size).clamp(1, 1000);
    let offset = page * bounded_page_size;
    let headers = build_skillssh_headers()?;
    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        urlencoding::encode(query),
        bounded_limit
    );

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .map_err(|error| format!("Failed to search skills.sh: {error}"))?
        .json::<Value>()
        .map_err(|error| format!("Failed to parse skills.sh search response: {error}"))?;

    if let Some(array) = response.as_array() {
        let all_items = parse_skills_array(array);
        let page_items = slice_page_items(&all_items, offset, bounded_page_size);
        return Ok(SkillsShSkillListRecord {
            has_more: all_items.len() > offset + page_items.len(),
            items: page_items,
            total_skills: None,
            page,
        });
    }

    parse_skill_list_response(&response, page).map(|result| {
        let page_items = slice_page_items(&result.items, offset, bounded_page_size);
        let has_more = result
            .total_skills
            .map(|total| total as usize > offset + page_items.len())
            .unwrap_or_else(|| result.items.len() > offset + page_items.len());

        SkillsShSkillListRecord {
            items: page_items,
            total_skills: result.total_skills,
            has_more,
            page,
        }
    })
}

pub fn fetch_skill_detail(
    cache_root_dir: &Path,
    source: &str,
    skill_id: &str,
) -> Result<SkillsShSkillDetailRecord, String> {
    if let Some(cached) =
        marketplace_cache_store::load_skill_detail(cache_root_dir, source, skill_id)?
    {
        if marketplace_cache_store::is_cache_fresh(&cached, SKILL_DETAIL_CACHE_TTL_SECS) {
            let markdown = cached.markdown;
            let raw_markdown = if cached.raw_markdown.is_empty() {
                markdown.clone()
            } else {
                cached.raw_markdown
            };
            return Ok(SkillsShSkillDetailRecord {
                description: cached.description,
                markdown,
                raw_markdown,
            });
        }
    }

    let client = build_http_client()?;

    if let Ok(Some(bundle)) = try_fetch_skill_bundle_from_download(&client, source, skill_id) {
        if let Some(detail) = detail_from_bundle(&bundle) {
            if !detail.markdown.trim().is_empty() {
                let _ = marketplace_cache_store::save_skill_detail(
                    cache_root_dir,
                    source,
                    skill_id,
                    &detail.description,
                    &detail.markdown,
                    &detail.raw_markdown,
                );
            }

            return Ok(detail);
        }
    }

    let (owner, repo) = parse_github_source(source)?;
    let branches = resolve_candidate_branches(&client, owner, repo);

    if let Some(raw_markdown) = try_fetch_skill_markdown(&client, owner, repo, skill_id, &branches)?
    {
        let detail = detail_from_raw_markdown(&raw_markdown);

        if !detail.markdown.trim().is_empty() {
            let _ = marketplace_cache_store::save_skill_detail(
                cache_root_dir,
                source,
                skill_id,
                &detail.description,
                &detail.markdown,
                &detail.raw_markdown,
            );
        }

        return Ok(detail);
    }

    if let Some(cached) =
        marketplace_cache_store::load_skill_detail(cache_root_dir, source, skill_id)?
    {
        let markdown = cached.markdown;
        let raw_markdown = if cached.raw_markdown.is_empty() {
            markdown.clone()
        } else {
            cached.raw_markdown
        };
        return Ok(SkillsShSkillDetailRecord {
            description: cached.description,
            markdown,
            raw_markdown,
        });
    }

    Ok(SkillsShSkillDetailRecord {
        description: String::new(),
        markdown: String::new(),
        raw_markdown: String::new(),
    })
}

pub fn fetch_skill_bundle(
    _cache_root_dir: &Path,
    source: &str,
    skill_id: &str,
    install_method: MarketplaceInstallMethod,
) -> Result<SkillsShSkillBundleRecord, String> {
    let client = build_http_client()?;

    if install_method == MarketplaceInstallMethod::SkillsSh {
        if let Ok(Some(bundle)) = try_fetch_skill_bundle_from_download(&client, source, skill_id) {
            return Ok(bundle);
        }
    }

    let (owner, repo) = parse_github_source(source)?;
    let branches = resolve_candidate_branches(&client, owner, repo);
    let resolved = resolve_skill_location(&client, owner, repo, skill_id, &branches)?
        .ok_or_else(|| format!("Failed to resolve remote skill bundle for {source}/{skill_id}"))?;
    let files = download_skill_files(
        &client,
        owner,
        repo,
        &resolved.branch,
        &resolved.skill_dir_path,
        &resolved.tree,
    )?;

    Ok(SkillsShSkillBundleRecord { files })
}

fn try_fetch_skill_bundle_from_download(
    client: &reqwest::blocking::Client,
    source: &str,
    skill_id: &str,
) -> Result<Option<SkillsShSkillBundleRecord>, String> {
    let (owner, repo) = parse_github_source(source)?;
    let headers = build_skillssh_headers()?;
    let url = format!(
        "https://skills.sh/api/download/{}/{}/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        urlencoding::encode(skill_id)
    );

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .map_err(|error| format!("Failed to fetch skills.sh download bundle: {error}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "skills.sh download request failed with status {}",
            response.status()
        ));
    }

    let payload = response
        .json::<SkillsShDownloadResponse>()
        .map_err(|error| format!("Failed to parse skills.sh download response: {error}"))?;

    let files = payload
        .files
        .into_iter()
        .map(|file| SkillsShSkillFileRecord {
            relative_path: file.path.replace('\\', "/"),
            contents: file.contents.into_bytes(),
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(SkillsShSkillBundleRecord { files }))
}

fn detail_from_bundle(bundle: &SkillsShSkillBundleRecord) -> Option<SkillsShSkillDetailRecord> {
    let raw_markdown = bundle
        .files
        .iter()
        .find(|file| file.relative_path == "SKILL.md" || file.relative_path.ends_with("/SKILL.md"))
        .map(|file| String::from_utf8_lossy(&file.contents).into_owned())?;

    Some(detail_from_raw_markdown(&raw_markdown))
}

fn detail_from_raw_markdown(raw_markdown: &str) -> SkillsShSkillDetailRecord {
    let (frontmatter_raw, markdown_body) = split_frontmatter(raw_markdown);
    let frontmatter = frontmatter_raw
        .as_ref()
        .and_then(|raw| serde_yaml::from_str::<Value>(raw).ok());
    let summary = summary_from_markdown(&markdown_body);
    let description =
        resolved_description(frontmatter.as_ref(), frontmatter_raw.as_deref(), &summary);

    SkillsShSkillDetailRecord {
        description,
        markdown: markdown_body,
        raw_markdown: raw_markdown.to_string(),
    }
}

fn parse_github_source(source: &str) -> Result<(&str, &str), String> {
    let mut parts = source.split('/');
    let owner = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Invalid GitHub source: {source}"))?;
    let repo = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Invalid GitHub source: {source}"))?;

    if parts.next().is_some() {
        return Err(format!("Invalid GitHub source: {source}"));
    }

    Ok((owner, repo))
}

fn resolve_candidate_branches(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
) -> Vec<String> {
    let mut branches = Vec::new();

    if let Ok(default_branch) = fetch_default_branch(client, owner, repo) {
        branches.push(default_branch);
    }

    if !branches.iter().any(|branch| branch == "main") {
        branches.push("main".to_string());
    }

    if !branches.iter().any(|branch| branch == "master") {
        branches.push("master".to_string());
    }

    branches
}

fn fetch_default_branch(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|error| format!("Failed to fetch GitHub repository metadata: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub repository metadata request failed with status {}",
            response.status()
        ));
    }

    response
        .json::<GitHubRepoInfo>()
        .map(|repo_info| repo_info.default_branch)
        .map_err(|error| format!("Failed to parse GitHub repository metadata: {error}"))
}

fn try_fetch_skill_markdown(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    skill_id: &str,
    branches: &[String],
) -> Result<Option<String>, String> {
    let mut candidate_skill_ids = vec![skill_id.to_string()];
    let simplified_skill_id = simplify_skill_id(skill_id);
    if simplified_skill_id != skill_id {
        candidate_skill_ids.push(simplified_skill_id);
    }

    const SKILL_PATH_PATTERNS: &[&str] = &[
        "skills/{skillId}/SKILL.md",
        "{skillId}/SKILL.md",
        ".skills/{skillId}/SKILL.md",
        "agent-skills/{skillId}/SKILL.md",
    ];

    for branch in branches {
        for candidate_skill_id in &candidate_skill_ids {
            for pattern in SKILL_PATH_PATTERNS {
                let path = pattern.replace("{skillId}", candidate_skill_id);
                if let Some(markdown) =
                    fetch_raw_github_markdown(client, owner, repo, branch, &path)?
                {
                    return Ok(Some(strip_frontmatter(&markdown)));
                }
            }
        }
    }

    for branch in branches {
        if let Some(path) = find_skill_markdown_path(client, owner, repo, branch, skill_id)? {
            if let Some(markdown) = fetch_raw_github_markdown(client, owner, repo, branch, &path)? {
                return Ok(Some(strip_frontmatter(&markdown)));
            }
        }
    }

    Ok(None)
}

fn resolve_skill_location(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    skill_id: &str,
    branches: &[String],
) -> Result<Option<ResolvedSkillLocation>, String> {
    for branch in branches {
        let Some(tree) = fetch_github_tree(client, owner, repo, branch)? else {
            continue;
        };
        let Some(skill_markdown_path) = resolve_skill_markdown_path_from_tree(&tree, skill_id)
        else {
            continue;
        };
        let Some(skill_dir_path) = skill_markdown_path
            .rsplit_once('/')
            .map(|(parent, _)| parent.to_string())
        else {
            continue;
        };

        return Ok(Some(ResolvedSkillLocation {
            branch: branch.clone(),
            skill_dir_path,
            tree,
        }));
    }

    Ok(None)
}

fn fetch_raw_github_markdown(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> Result<Option<String>, String> {
    let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}");
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Failed to fetch GitHub raw content: {error}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub raw content request failed with status {}",
            response.status()
        ));
    }

    response
        .text()
        .map(Some)
        .map_err(|error| format!("Failed to read GitHub raw content: {error}"))
}

fn fetch_raw_github_file(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> Result<Option<Vec<u8>>, String> {
    let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}");
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Failed to fetch GitHub raw content: {error}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub raw content request failed with status {}",
            response.status()
        ));
    }

    response
        .bytes()
        .map(|bytes| Some(bytes.to_vec()))
        .map_err(|error| format!("Failed to read GitHub raw content: {error}"))
}

fn fetch_github_tree(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Option<Vec<GitHubTreeEntry>>, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/git/trees/{branch}?recursive=1");
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|error| format!("Failed to fetch GitHub repository tree: {error}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "GitHub repository tree request failed with status {}",
            response.status()
        ));
    }

    response
        .json::<GitHubTreeResponse>()
        .map(|tree| Some(tree.tree))
        .map_err(|error| format!("Failed to parse GitHub repository tree: {error}"))
}

fn find_skill_markdown_path(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    branch: &str,
    skill_id: &str,
) -> Result<Option<String>, String> {
    let Some(tree) = fetch_github_tree(client, owner, repo, branch)? else {
        return Ok(None);
    };

    Ok(resolve_skill_markdown_path_from_tree(&tree, skill_id))
}

fn resolve_skill_markdown_path_from_tree(
    tree: &[GitHubTreeEntry],
    skill_id: &str,
) -> Option<String> {
    let skill_paths = tree
        .iter()
        .filter(|entry| entry.entry_type == "blob" && entry.path.ends_with("/SKILL.md"))
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();

    if skill_paths.is_empty() {
        return None;
    }

    if let Some(path) = match_skill_path(&skill_paths, skill_id) {
        return Some(path);
    }

    let simplified_skill_id = simplify_skill_id(skill_id);
    if simplified_skill_id != skill_id {
        if let Some(path) = match_skill_path(&skill_paths, &simplified_skill_id) {
            return Some(path);
        }
    }

    skill_paths.into_iter().next()
}

fn download_skill_files(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    branch: &str,
    skill_dir_path: &str,
    tree: &[GitHubTreeEntry],
) -> Result<Vec<SkillsShSkillFileRecord>, String> {
    let prefix = format!("{skill_dir_path}/");
    let mut skill_files = tree
        .iter()
        .filter(|entry| entry.entry_type == "blob" && entry.path.starts_with(&prefix))
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    skill_files.sort();

    let mut files = Vec::with_capacity(skill_files.len());
    for path in skill_files {
        let Some(relative_path) = path.strip_prefix(&prefix) else {
            continue;
        };
        let contents = fetch_raw_github_file(client, owner, repo, branch, &path)?
            .ok_or_else(|| format!("Remote file disappeared during install: {path}"))?;
        files.push(SkillsShSkillFileRecord {
            relative_path: relative_path.replace('\\', "/"),
            contents,
        });
    }

    Ok(files)
}

fn match_skill_path(paths: &[String], skill_id: &str) -> Option<String> {
    paths.iter().find_map(|path| {
        let parent_dir = path.rsplit_once('/').map(|(dir, _)| dir)?;
        let dir_name = parent_dir.rsplit('/').next().unwrap_or(parent_dir);
        (dir_name == skill_id).then(|| path.clone())
    })
}

fn simplify_skill_id(skill_id: &str) -> String {
    let Some((_, rest)) = skill_id.split_once('-') else {
        return skill_id.to_string();
    };

    if rest.is_empty() {
        skill_id.to_string()
    } else {
        rest.to_string()
    }
}

fn split_frontmatter(contents: &str) -> (Option<String>, String) {
    let normalized = contents.replace("\r\n", "\n");
    if !normalized.starts_with("---\n") {
        return (None, normalized);
    }

    let remainder = &normalized[4..];
    if let Some(index) = remainder.find("\n---\n") {
        let frontmatter = remainder[..index].to_string();
        let body = remainder[index + 5..].to_string();
        (Some(frontmatter), body)
    } else {
        (None, normalized)
    }
}

fn summary_from_markdown(markdown: &str) -> String {
    markdown
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.chars().take(140).collect())
        .unwrap_or_else(|| "Marketplace skill".into())
}

fn description_from_frontmatter(frontmatter: Option<&Value>) -> Option<String> {
    frontmatter
        .and_then(|value| value.get("description"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn description_from_frontmatter_raw(frontmatter_raw: Option<&str>) -> Option<String> {
    let mut lines = frontmatter_raw?.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || line != trimmed {
            continue;
        }

        let Some(raw_value) = trimmed.strip_prefix("description:") else {
            continue;
        };

        let value = raw_value.trim();
        if value.is_empty() {
            continue;
        }

        if let Some(block_indicator) = value.chars().next().filter(|ch| *ch == '>' || *ch == '|') {
            let fold_lines = block_indicator == '>';
            let mut collected = Vec::new();

            while let Some(next_line) = lines.peek() {
                if next_line.trim().is_empty() {
                    lines.next();
                    continue;
                }

                if !next_line.starts_with(' ') && !next_line.starts_with('\t') {
                    break;
                }

                collected.push(next_line.trim().to_string());
                lines.next();
            }

            if collected.is_empty() {
                return None;
            }

            return Some(if fold_lines {
                collected.join(" ")
            } else {
                collected.join("\n")
            });
        }

        return Some(unquote_yaml_value(value));
    }

    None
}

fn unquote_yaml_value(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }

    value.to_string()
}

fn resolved_description(
    frontmatter: Option<&Value>,
    frontmatter_raw: Option<&str>,
    fallback: &str,
) -> String {
    description_from_frontmatter(frontmatter)
        .or_else(|| description_from_frontmatter_raw(frontmatter_raw))
        .unwrap_or_else(|| fallback.to_string())
}

fn strip_frontmatter(markdown: &str) -> String {
    let normalized = markdown.replace("\r\n", "\n");
    let Some(rest) = normalized.strip_prefix("---\n") else {
        return normalized.trim().to_string();
    };
    let Some((_, content)) = rest.split_once("\n---\n") else {
        return normalized.trim().to_string();
    };

    content.trim().to_string()
}

fn parse_total_skills_value(value: &Value) -> Option<u64> {
    value
        .get("totalSkills")
        .and_then(value_to_u64)
        .or_else(|| value.get("total").and_then(value_to_u64))
        .or_else(|| value.get("count").and_then(value_to_u64))
}

fn parse_skill_list_response(
    response: &Value,
    page: usize,
) -> Result<SkillsShSkillListRecord, String> {
    if let Some(array) = response.get("skills").and_then(Value::as_array) {
        let items = parse_skills_array(array);
        let total_skills = parse_total_skills_value(response).or(Some(items.len() as u64));
        return Ok(SkillsShSkillListRecord {
            has_more: response
                .get("hasMore")
                .and_then(Value::as_bool)
                .unwrap_or_else(|| {
                    total_skills
                        .map(|total| total as usize > (page + 1) * MARKETPLACE_PAGE_SIZE)
                        .unwrap_or(false)
                }),
            items,
            total_skills,
            page,
        });
    }

    if let Some(array) = response.get("items").and_then(Value::as_array) {
        let items = parse_skills_array(array);
        let total_skills = parse_total_skills_value(response).or(Some(items.len() as u64));
        return Ok(SkillsShSkillListRecord {
            has_more: response
                .get("hasMore")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            items,
            total_skills,
            page,
        });
    }

    if response.is_object() {
        return Ok(SkillsShSkillListRecord {
            items: Vec::new(),
            total_skills: parse_total_skills_value(response),
            has_more: response
                .get("hasMore")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            page,
        });
    }

    Err("Unsupported skills.sh list response format.".into())
}

fn value_to_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(parse_total_skills_text))
}

fn slice_page_items(
    items: &[SkillsShSkillRecord],
    offset: usize,
    page_size: usize,
) -> Vec<SkillsShSkillRecord> {
    items.iter().skip(offset).take(page_size).cloned().collect()
}

fn parse_total_skills_text(value: &str) -> Option<u64> {
    value.replace(',', "").trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{
        detail_from_bundle, parse_skill_list_response, parse_skills_array,
        resolve_skill_markdown_path_from_tree, simplify_skill_id, strip_frontmatter,
        GitHubTreeEntry, SkillsShSkillBundleRecord, SkillsShSkillFileRecord,
    };
    use serde_json::json;

    fn blob(path: &str) -> GitHubTreeEntry {
        GitHubTreeEntry {
            path: path.into(),
            entry_type: "blob".into(),
        }
    }

    #[test]
    fn resolve_skill_markdown_path_from_tree_prefers_matching_directory_name() {
        let tree = vec![
            blob("skills/other/SKILL.md"),
            blob("skills/playwright/SKILL.md"),
            blob("skills/playwright/scripts/install.sh"),
        ];

        assert_eq!(
            resolve_skill_markdown_path_from_tree(&tree, "playwright"),
            Some("skills/playwright/SKILL.md".into())
        );
    }

    #[test]
    fn resolve_skill_markdown_path_from_tree_supports_simplified_skill_ids() {
        let tree = vec![blob("skills/react/SKILL.md")];

        assert_eq!(
            resolve_skill_markdown_path_from_tree(&tree, "openai-react"),
            Some("skills/react/SKILL.md".into())
        );
    }

    #[test]
    fn simplify_skill_id_keeps_unsplittable_values() {
        assert_eq!(simplify_skill_id("playwright"), "playwright");
    }

    #[test]
    fn strip_frontmatter_returns_trimmed_markdown_body() {
        let markdown = "---\nname: Demo\n---\n\n# Title\n";
        assert_eq!(strip_frontmatter(markdown), "# Title");
    }

    #[test]
    fn detail_from_bundle_reads_root_skill_markdown() {
        let bundle = SkillsShSkillBundleRecord {
            files: vec![SkillsShSkillFileRecord {
                relative_path: "SKILL.md".into(),
                contents: b"---\ndescription: Demo skill\n---\n\n# Heading\nBody".to_vec(),
            }],
        };

        let detail = detail_from_bundle(&bundle).expect("detail from bundle");
        assert_eq!(detail.description, "Demo skill");
        assert_eq!(detail.markdown, "\n# Heading\nBody");
    }

    #[test]
    fn detail_from_bundle_ignores_non_entry_files() {
        let bundle = SkillsShSkillBundleRecord {
            files: vec![SkillsShSkillFileRecord {
                relative_path: "notes.txt".into(),
                contents: b"not a skill".to_vec(),
            }],
        };

        assert!(detail_from_bundle(&bundle).is_none());
    }

    #[test]
    fn parse_skills_array_skips_duplicate_items() {
        let items = vec![
            json!({
                "source": "skillssh/skills",
                "skillId": "ai-image-generation",
                "name": "ai-image-generation",
                "installs": 14963
            }),
            json!({
                "source": "jackiexiao/jackie-skills-starter",
                "skillId": "ai-image-generation",
                "name": "ai-image-generation",
                "installs": 53,
                "isDuplicate": true
            }),
        ];

        let skills = parse_skills_array(&items);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source, "skillssh/skills");
        assert_eq!(skills[0].skill_id, "ai-image-generation");
    }

    #[test]
    fn parse_skill_list_response_reads_total_skills() {
        let response = json!({
            "skills": [
                {
                    "source": "skillssh/skills",
                    "skillId": "demo",
                    "name": "Demo",
                    "installs": 12
                }
            ],
            "total": 91007
        });

        let parsed =
            parse_skill_list_response(&response, 0).expect("expected skill list response to parse");

        assert_eq!(parsed.total_skills, Some(91007));
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].skill_id, "demo");
        assert!(parsed.has_more);
    }

    #[test]
    fn parse_skill_list_response_supports_items_key() {
        let response = json!({
            "items": [
                {
                    "source": "skillssh/skills",
                    "skill_id": "demo",
                    "name": "Demo",
                    "installs": 12
                }
            ],
            "count": "91,007"
        });

        let parsed =
            parse_skill_list_response(&response, 0).expect("expected items response to parse");

        assert_eq!(parsed.total_skills, Some(91007));
        assert_eq!(parsed.items.len(), 1);
    }
}

fn parse_skills_array(items: &[Value]) -> Vec<SkillsShSkillRecord> {
    let mut seen = HashSet::new();
    let mut skills = Vec::new();

    for item in items {
        if item
            .get("isDuplicate")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }

        let source = item
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let skill_id = item
            .get("skillId")
            .or_else(|| item.get("skill_id"))
            .or_else(|| item.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        if source.is_empty() || skill_id.is_empty() {
            continue;
        }

        let id = format!("{source}/{skill_id}");
        if !seen.insert(id.clone()) {
            continue;
        }

        let name = item
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(&skill_id)
            .to_string();
        let installs = item.get("installs").and_then(Value::as_u64).unwrap_or(0);

        skills.push(SkillsShSkillRecord {
            id,
            skill_id,
            name,
            source,
            installs,
        });
    }

    skills
}
