// Tauri IPC types — совпадают с Rust commands
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

export interface ScenarioStep {
  type: string;
  config: Record<string, string>;
}

export interface ExecutionResult {
  success: boolean;
  log: string[];
}

export async function executeScenario(steps: ScenarioStep[]): Promise<ExecutionResult> {
  return invoke<ExecutionResult>('execute_scenario', { steps });
}

export async function stopExecution(): Promise<void> {
  return invoke<void>('stop_execution', {});
}

export async function highlightElement(selector: string): Promise<void> {
  return invoke<void>('highlight_element', { selector });
}

// ─── Project management ───────────────────────────────────────

export interface ProjectInfo {
  name: string;
  file_name: string;
}

export async function listProjects(): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>('list_projects', {});
}

export async function saveProject(project: Record<string, unknown>): Promise<void> {
  return invoke<void>('save_project', { projectJson: project });
}

export async function loadProject(fileName: string): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('load_project', { fileName });
}

export async function deleteProject(fileName: string): Promise<void> {
  return invoke<void>('delete_project', { fileName });
}

// ─── File dialogs ─────────────────────────────────────────────

export async function openProjectFile(): Promise<string | null> {
  const selected = await open({
    title: 'Открыть проект',
    filters: [{ name: 'RPA Project', extensions: ['json'] }],
    multiple: false,
  });
  return typeof selected === 'string' ? selected : null;
}

export async function loadProjectFromPath(filePath: string): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('load_project_from_path', { filePath });
}

export async function saveProjectFile(project: Record<string, unknown>, defaultName: string): Promise<string | null> {
  const path = await save({
    title: 'Сохранить проект',
    defaultPath: `${defaultName}.json`,
    filters: [{ name: 'RPA Project', extensions: ['json'] }],
  });
  if (!path) return null;
  await invoke<void>('save_project_to_path', { projectJson: project, path });
  return path;
}
