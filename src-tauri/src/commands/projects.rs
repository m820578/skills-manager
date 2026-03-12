use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::core::project_scanner;
use crate::core::skill_store::{ProjectRecord, SkillStore};

#[derive(Serialize)]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub sort_order: i32,
    pub skill_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct ProjectSkillDocumentDto {
    pub skill_name: String,
    pub filename: String,
    pub content: String,
}

fn project_to_dto(rec: &ProjectRecord) -> ProjectDto {
    let skill_count = project_scanner::read_project_skills(Path::new(&rec.path)).len();
    ProjectDto {
        id: rec.id.clone(),
        name: rec.name.clone(),
        path: rec.path.clone(),
        sort_order: rec.sort_order,
        skill_count,
        created_at: rec.created_at,
        updated_at: rec.updated_at,
    }
}

#[tauri::command]
pub fn get_projects(store: State<'_, Arc<SkillStore>>) -> Result<Vec<ProjectDto>, String> {
    let records = store.get_all_projects().map_err(|e| e.to_string())?;
    Ok(records.iter().map(project_to_dto).collect())
}

#[tauri::command]
pub fn add_project(store: State<'_, Arc<SkillStore>>, path: String) -> Result<ProjectDto, String> {
    let project_path = Path::new(&path);
    let skills_dir = project_path.join(".claude").join("skills");
    if !skills_dir.is_dir() {
        return Err("Directory does not contain .claude/skills/".to_string());
    }

    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let now = chrono::Utc::now().timestamp_millis();
    let record = ProjectRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path.clone(),
        sort_order: 0,
        created_at: now,
        updated_at: now,
    };

    store.insert_project(&record).map_err(|e| e.to_string())?;
    Ok(project_to_dto(&record))
}

#[tauri::command]
pub fn remove_project(store: State<'_, Arc<SkillStore>>, id: String) -> Result<(), String> {
    store.delete_project(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_projects(root: String) -> Result<Vec<String>, String> {
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err("Directory does not exist".to_string());
    }
    Ok(project_scanner::scan_projects_in_dir(root_path, 4))
}

#[tauri::command]
pub fn get_project_skills(
    store: State<'_, Arc<SkillStore>>,
    project_id: String,
) -> Result<Vec<project_scanner::ProjectSkillInfo>, String> {
    let record = store
        .get_project_by_id(&project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;
    Ok(project_scanner::read_project_skills(Path::new(&record.path)))
}

#[tauri::command]
pub fn get_project_skill_document(
    project_path: String,
    skill_name: String,
) -> Result<ProjectSkillDocumentDto, String> {
    let skill_dir = Path::new(&project_path)
        .join(".claude")
        .join("skills")
        .join(&skill_name);

    if !skill_dir.is_dir() {
        return Err("Skill directory not found".to_string());
    }

    let candidates = ["SKILL.md", "skill.md", "CLAUDE.md", "README.md"];
    for candidate in &candidates {
        let file_path = skill_dir.join(candidate);
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
            return Ok(ProjectSkillDocumentDto {
                skill_name,
                filename: candidate.to_string(),
                content,
            });
        }
    }

    Err("No document file found in skill directory".to_string())
}
