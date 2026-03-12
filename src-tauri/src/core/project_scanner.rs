use serde::Serialize;
use std::path::Path;

use super::skill_metadata;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSkillInfo {
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub files: Vec<String>,
}

/// Read all skills under `<project_path>/.claude/skills/`.
pub fn read_project_skills(project_path: &Path) -> Vec<ProjectSkillInfo> {
    let skills_dir = project_path.join(".claude").join("skills");
    if !skills_dir.is_dir() {
        return vec![];
    }

    let mut skills = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&skills_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta = skill_metadata::parse_skill_md(&path);
            let name = meta
                .name
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                });

            let files = list_files(&path);

            skills.push(ProjectSkillInfo {
                name,
                description: meta.description,
                path: path.to_string_lossy().to_string(),
                files,
            });
        }
    }

    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    skills
}

/// Scan a root directory for projects containing `.claude/skills/`.
pub fn scan_projects_in_dir(root: &Path, max_depth: usize) -> Vec<String> {
    let mut results = Vec::new();
    scan_recursive(root, 0, max_depth, &mut results);
    results.sort();
    results
}

fn scan_recursive(dir: &Path, depth: usize, max_depth: usize, results: &mut Vec<String>) {
    if depth > max_depth {
        return;
    }

    let claude_skills = dir.join(".claude").join("skills");
    if claude_skills.is_dir() {
        results.push(dir.to_string_lossy().to_string());
        return; // don't recurse into subdirectories of a matched project
    }

    if depth == max_depth {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Skip hidden directories and common non-project dirs
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "__pycache__" {
                continue;
            }
            scan_recursive(&path, depth + 1, max_depth, results);
        }
    }
}

fn list_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    files
}
