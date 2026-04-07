export type BlockType =
  | 'Start'
  | 'LaunchApp' | 'CloseApp'
  | 'Click' | 'DoubleClick' | 'RightClick' | 'MoveMouse' | 'DragAndDrop'
  | 'TypeText' | 'KeyPress'
  | 'ExtractText' | 'Screenshot'
  | 'Wait' | 'WaitForElement' | 'Retry' | 'Condition'
  | 'ReadFile' | 'WriteFile';

export interface BlockConfig {
  [key: string]: string;
}

export interface BlockNodeData {
  blockType: BlockType;
  config: BlockConfig;
  [key: string]: unknown;
}

export interface FlowBlock {
  id: string;
  blockType: BlockType;
  position: { x: number; y: number };
  config: BlockConfig;
}

export interface Connection {
  fromId: string;
  toId: string;
}

// ─── Project / Diagram ───────────────────────────────────────

export interface Diagram {
  id: string;
  name: string;
  nodes: SerializedNode[];
  edges: SerializedEdge[];
}

export interface SerializedNode {
  id: string;
  blockType: string;
  position: { x: number; y: number };
  config: Record<string, string>;
}

export interface SerializedEdge {
  id: string;
  source: string;
  target: string;
}

export interface Project {
  name: string;
  diagrams: Diagram[];
  activeDiagramId: string;
}

// ─── Helpers ──────────────────────────────────────────────────

export const BLOCK_LABELS: Record<BlockType, string> = {
  Start: 'Старт',
  LaunchApp: 'Запуск приложения',
  CloseApp: 'Закрытие приложения',
  Click: 'Клик',
  DoubleClick: 'Двойной клик',
  RightClick: 'Правый клик',
  MoveMouse: 'Переместить мышь',
  DragAndDrop: 'Перетаскивание',
  TypeText: 'Ввод текста',
  KeyPress: 'Нажатие клавиш',
  ExtractText: 'Извлечение текста',
  Screenshot: 'Скриншот',
  Wait: 'Пауза',
  WaitForElement: 'Ожидание элемента',
  Retry: 'Повтор',
  Condition: 'Условие',
  ReadFile: 'Чтение файла',
  WriteFile: 'Запись файла',
};

export const BLOCK_ICONS: Record<BlockType, string> = {
  Start: '▶️', LaunchApp: '🚀', CloseApp: '🛑',
  Click: '🖱', DoubleClick: '🖱🖱', RightClick: '🖱R', MoveMouse: '🔹', DragAndDrop: '↔️',
  TypeText: '⌨', KeyPress: '⌨️',
  ExtractText: '📄', Screenshot: '📸',
  Wait: '⏳', WaitForElement: '⏱', Retry: '🔄', Condition: '🔍',
  ReadFile: '📖', WriteFile: '📝',
};

export const BLOCK_ACCENT: Record<BlockType, string> = {
  Start: '#4CAF50', LaunchApp: '#4CAF50', CloseApp: '#F44336',
  Click: '#2196F3', DoubleClick: '#2196F3', RightClick: '#2196F3', MoveMouse: '#03A9F4', DragAndDrop: '#00BCD4',
  TypeText: '#FF9800', KeyPress: '#FF9800',
  ExtractText: '#9C27B0', Screenshot: '#673AB7',
  Wait: '#795548', WaitForElement: '#795548', Retry: '#607D8B', Condition: '#FFC107',
  ReadFile: '#4CAF50', WriteFile: '#8BC34A',
};

/// Общий pid-хелпер — добавляет поле pid (число или имя переменной)
function withPid(cfg: Record<string, string>): Record<string, string> {
  return { ...cfg, pid: '' };
}

export function createDefaultConfig(blockType: BlockType): BlockConfig {
  switch (blockType) {
    case 'Start': return {};
    case 'LaunchApp': return { app: 'notepad', var_name: '_last_pid' };
    case 'CloseApp': return { process_name: 'notepad', force: 'false', pid: '' };
    case 'Click': return withPid({ selector: 'classname=Edit' });
    case 'DoubleClick': return withPid({ selector: 'classname=Edit' });
    case 'RightClick': return withPid({ selector: 'classname=Edit' });
    case 'MoveMouse': return withPid({ selector: 'classname=Edit' });
    case 'DragAndDrop': return withPid({ selector: 'classname=Edit', target_selector: 'classname=Edit', delay_ms: '500' });
    case 'TypeText': return withPid({ selector: 'classname=Edit', text: '' });
    case 'KeyPress': return { keys: '{Enter}', delay_ms: '42' };
    case 'ExtractText': return withPid({ selector: 'classname=Edit', var_name: 'extracted_text' });
    case 'Screenshot': return withPid({ selector: '', output_path: 'screenshot.bmp' });
    case 'Wait': return { duration_ms: '1000' };
    case 'WaitForElement': return withPid({ selector: 'classname=Edit', timeout_ms: '10000', interval_ms: '500' });
    case 'Retry': return withPid({ selector: 'classname=Edit', max_attempts: '3', delay_ms: '1000' });
    case 'Condition': return withPid({ selector: 'classname=Edit', var_name: 'condition_result' });
    case 'ReadFile': return { file_path: '', var_name: 'file_content' };
    case 'WriteFile': return { file_path: '', content: '', append: 'false' };
  }
}

export function blockTypeToToolName(blockType: BlockType): string {
  switch (blockType) {
    case 'Start': return 'Start';
    case 'LaunchApp': return 'LaunchApp';
    case 'CloseApp': return 'CloseApp';
    case 'Click': return 'Click';
    case 'DoubleClick': return 'DoubleClick';
    case 'RightClick': return 'RightClick';
    case 'MoveMouse': return 'MoveMouse';
    case 'DragAndDrop': return 'DragAndDrop';
    case 'TypeText': return 'TypeText';
    case 'KeyPress': return 'KeyPress';
    case 'ExtractText': return 'ExtractText';
    case 'Screenshot': return 'Screenshot';
    case 'Wait': return 'Wait';
    case 'WaitForElement': return 'WaitForElement';
    case 'Retry': return 'Retry';
    case 'Condition': return 'Condition';
    case 'ReadFile': return 'ReadFile';
    case 'WriteFile': return 'WriteFile';
  }
}

// ─── Project Factory ─────────────────────────────────────────

export function createDiagram(name: string): Diagram {
  const id = crypto.randomUUID();
  const startId = crypto.randomUUID();
  return {
    id,
    name,
    nodes: [{
      id: startId,
      blockType: 'Start',
      position: { x: 100, y: 80 },
      config: {},
    }],
    edges: [],
  };
}

export function createProject(name: string): Project {
  const main = createDiagram('Main');
  return {
    name,
    diagrams: [main],
    activeDiagramId: main.id,
  };
}
