import { BlockType } from '../types';

// Модульный singleton для хранения перетаскиваемого типа блока
// Используется и в BlockPalette (запись) и в FlowCanvas (чтение)
let currentDragType: BlockType | null = null;

export const dragStore = {
  set: (bt: BlockType | null) => {
    currentDragType = bt;
  },
  get: () => currentDragType,
};
