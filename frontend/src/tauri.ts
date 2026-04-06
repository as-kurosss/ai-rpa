// Tauri IPC types — совпадают с Rust commands
import { invoke } from '@tauri-apps/api/core';

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
