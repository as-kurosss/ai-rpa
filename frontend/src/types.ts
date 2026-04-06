export type BlockType = 'LaunchApp' | 'Click' | 'TypeText';

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
  Click: 'Клик',
  TypeText: 'Ввод текста',
};

export const BLOCK_ICONS: Record<BlockType, string> = {
  LaunchApp: '🚀',
  Click: '🖱',
  TypeText: '⌨',
};

export const BLOCK_ACCENT: Record<BlockType, string> = {
  LaunchApp: '#4CAF50', // green
  Click: '#2196F3',     // blue
  TypeText: '#FF9800',  // orange
};

export function createDefaultConfig(blockType: BlockType): BlockConfig {
  switch (blockType) {
    case 'LaunchApp':
      return { app: 'notepad' };
    case 'Click':
      return { selector: 'classname=Edit' };
    case 'TypeText':
      return { selector: 'classname=Edit', text: '' };
  }
}

export function blockTypeToToolName(blockType: BlockType): string {
  switch (blockType) {
    case 'LaunchApp': return 'LaunchApp';
    case 'Click': return 'Click';
    case 'TypeText': return 'TypeText';
  }
}
