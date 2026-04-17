use serde_json::Value;

pub fn split_frontmatter(contents: &str) -> (Option<String>, String) {
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

pub fn summary_from_markdown(markdown: &str, fallback: &str) -> String {
    markdown
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.chars().take(140).collect())
        .unwrap_or_else(|| fallback.to_string())
}

pub fn description_from_frontmatter(frontmatter: Option<&Value>) -> Option<String> {
    frontmatter
        .and_then(|value| value.get("description"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

pub fn description_from_frontmatter_raw(frontmatter_raw: Option<&str>) -> Option<String> {
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
            return None;
        }

        if matches!(value, "|" | ">" | "|-" | ">-" | "|+" | ">+") {
            let mut block_lines = Vec::new();

            while let Some(next_line) = lines.peek() {
                if next_line.trim().is_empty() {
                    block_lines.push(String::new());
                    lines.next();
                    continue;
                }

                if next_line.trim_start() == *next_line {
                    break;
                }

                block_lines.push(next_line.trim().to_string());
                lines.next();
            }

            let block_text = if value.starts_with('>') {
                block_lines.join(" ")
            } else {
                block_lines.join("\n")
            };
            let normalized = block_text.trim().to_string();
            return (!normalized.is_empty()).then_some(normalized);
        }

        let normalized = value
            .trim_matches(|character| matches!(character, '"' | '\''))
            .trim()
            .to_string();
        return (!normalized.is_empty()).then_some(normalized);
    }

    None
}

pub fn resolved_description(
    frontmatter: Option<&Value>,
    frontmatter_raw: Option<&str>,
    fallback: &str,
) -> String {
    description_from_frontmatter(frontmatter)
        .or_else(|| description_from_frontmatter_raw(frontmatter_raw))
        .unwrap_or_else(|| fallback.to_string())
}
