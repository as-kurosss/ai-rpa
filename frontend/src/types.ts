export type BlockType =
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

export const BLOCK_LABELS: Record<BlockType, string> = {
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
  LaunchApp: '🚀', CloseApp: '🛑',
  Click: '🖱', DoubleClick: '🖱🖱', RightClick: '🖱R', MoveMouse: '🔹', DragAndDrop: '↔️',
  TypeText: '⌨', KeyPress: '⌨️',
  ExtractText: '📄', Screenshot: '📸',
  Wait: '⏳', WaitForElement: '⏱', Retry: '🔄', Condition: '🔍',
  ReadFile: '📖', WriteFile: '📝',
};

export const BLOCK_ACCENT: Record<BlockType, string> = {
  LaunchApp: '#4CAF50', CloseApp: '#F44336',
  Click: '#2196F3', DoubleClick: '#2196F3', RightClick: '#2196F3', MoveMouse: '#03A9F4', DragAndDrop: '#00BCD4',
  TypeText: '#FF9800', KeyPress: '#FF9800',
  ExtractText: '#9C27B0', Screenshot: '#673AB7',
  Wait: '#795548', WaitForElement: '#795548', Retry: '#607D8B', Condition: '#FFC107',
  ReadFile: '#4CAF50', WriteFile: '#8BC34A',
};

export function createDefaultConfig(blockType: BlockType): BlockConfig {
  switch (blockType) {
    case 'LaunchApp': return { app: 'notepad' };
    case 'CloseApp': return { process_name: 'notepad', force: 'false' };
    case 'Click': return { selector: 'classname=Edit' };
    case 'DoubleClick': return { selector: 'classname=Edit' };
    case 'RightClick': return { selector: 'classname=Edit' };
    case 'MoveMouse': return { selector: 'classname=Edit' };
    case 'DragAndDrop': return { selector: 'classname=Edit', target_selector: 'classname=Edit', delay_ms: '500' };
    case 'TypeText': return { selector: 'classname=Edit', text: '' };
    case 'KeyPress': return { keys: '{Enter}', delay_ms: '42' };
    case 'ExtractText': return { selector: 'classname=Edit', var_name: 'extracted_text' };
    case 'Screenshot': return { selector: '', output_path: 'screenshot.bmp' };
    case 'Wait': return { duration_ms: '1000' };
    case 'WaitForElement': return { selector: 'classname=Edit', timeout_ms: '10000', interval_ms: '500' };
    case 'Retry': return { selector: 'classname=Edit', max_attempts: '3', delay_ms: '1000' };
    case 'Condition': return { selector: 'classname=Edit', var_name: 'condition_result' };
    case 'ReadFile': return { file_path: '', var_name: 'file_content' };
    case 'WriteFile': return { file_path: '', content: '', append: 'false' };
  }
}

export function blockTypeToToolName(blockType: BlockType): string {
  switch (blockType) {
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
