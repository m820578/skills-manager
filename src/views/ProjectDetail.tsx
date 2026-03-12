import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { FolderOpen, FileText, ChevronDown, ChevronRight, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useApp } from "../context/AppContext";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import type { ProjectSkill } from "../lib/tauri";

export function ProjectDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const { projects } = useApp();
  const [skills, setSkills] = useState<ProjectSkill[]>([]);
  const [loading, setLoading] = useState(true);
  const [expandedSkill, setExpandedSkill] = useState<string | null>(null);
  const [docContent, setDocContent] = useState<string | null>(null);
  const [docLoading, setDocLoading] = useState(false);

  const project = projects.find((p) => p.id === id);

  const loadSkills = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    try {
      const result = await api.getProjectSkills(id);
      setSkills(result);
    } catch (e) {
      console.error("Failed to load project skills:", e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  useEffect(() => {
    if (!project && !loading) {
      navigate("/");
    }
  }, [project, loading, navigate]);

  const handleToggleSkill = async (skill: ProjectSkill) => {
    if (expandedSkill === skill.name) {
      setExpandedSkill(null);
      setDocContent(null);
      return;
    }

    setExpandedSkill(skill.name);
    setDocContent(null);
    setDocLoading(true);

    if (!project) return;
    try {
      const doc = await api.getProjectSkillDocument(project.path, skill.name);
      setDocContent(doc.content);
    } catch {
      setDocContent(null);
    } finally {
      setDocLoading(false);
    }
  };

  if (!project) return null;

  return (
    <div>
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center gap-2 mb-1">
          <FolderOpen className="w-5 h-5 text-accent" />
          <h1 className="text-lg font-semibold text-primary">{project.name}</h1>
        </div>
        <p className="text-[13px] text-muted font-mono truncate">{project.path}</p>
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-[13px] font-semibold text-secondary">
          {t("project.skills")} ({skills.length})
        </h2>
        <button
          onClick={loadSkills}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[13px] text-muted hover:text-secondary hover:bg-surface-hover transition-colors outline-none"
        >
          <RefreshCw className={cn("w-3.5 h-3.5", loading && "animate-spin")} />
        </button>
      </div>

      {/* Skills list */}
      {loading ? (
        <div className="py-12 text-center text-[13px] text-muted">
          {t("common.loading")}
        </div>
      ) : skills.length === 0 ? (
        <div className="py-12 text-center">
          <p className="text-[13px] text-muted mb-1">{t("project.noSkills")}</p>
          <p className="text-[12px] text-faint">{t("project.noSkillsHint")}</p>
        </div>
      ) : (
        <div className="space-y-1.5">
          {skills.map((skill) => {
            const isExpanded = expandedSkill === skill.name;
            return (
              <div
                key={skill.name}
                className="border border-border-subtle rounded-lg overflow-hidden"
              >
                <button
                  onClick={() => handleToggleSkill(skill)}
                  className="flex items-center gap-3 w-full px-4 py-3 text-left hover:bg-surface-hover transition-colors outline-none"
                >
                  {isExpanded ? (
                    <ChevronDown className="w-4 h-4 text-muted shrink-0" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-muted shrink-0" />
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="text-[13px] font-medium text-primary truncate">
                      {skill.name}
                    </div>
                    {skill.description && (
                      <div className="text-[12px] text-muted truncate mt-0.5">
                        {skill.description}
                      </div>
                    )}
                  </div>
                  {skill.files.length > 0 && (
                    <div className="flex items-center gap-1 text-[12px] text-faint shrink-0">
                      <FileText className="w-3 h-3" />
                      {skill.files.length}
                    </div>
                  )}
                </button>

                {isExpanded && (
                  <div className="border-t border-border-subtle">
                    {/* File list */}
                    {skill.files.length > 0 && (
                      <div className="px-4 py-2 bg-background/50">
                        <div className="text-[12px] text-muted mb-1.5">
                          {t("project.skillFiles")}
                        </div>
                        <div className="flex flex-wrap gap-1.5">
                          {skill.files.map((f) => (
                            <span
                              key={f}
                              className="px-2 py-0.5 rounded bg-surface-hover text-[12px] text-tertiary font-mono"
                            >
                              {f}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                    {/* Document content */}
                    {docLoading ? (
                      <div className="px-4 py-4 text-[13px] text-muted">
                        {t("common.loading")}
                      </div>
                    ) : docContent ? (
                      <div className="px-4 py-3 prose prose-sm dark:prose-invert max-w-none text-[13px] leading-relaxed">
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>
                          {stripFrontmatter(docContent)}
                        </ReactMarkdown>
                      </div>
                    ) : (
                      <div className="px-4 py-3 text-[12px] text-faint">
                        {t("common.documentMissing")}
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function stripFrontmatter(content: string): string {
  const trimmed = content.trim();
  if (!trimmed.startsWith("---")) return content;
  const rest = trimmed.slice(3);
  const end = rest.indexOf("---");
  if (end === -1) return content;
  return rest.slice(end + 3).trim();
}
