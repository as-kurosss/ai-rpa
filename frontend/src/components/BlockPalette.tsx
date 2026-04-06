import { useState } from 'react';
import { BlockType, BLOCK_LABELS, BLOCK_ICONS } from '../types';
import { dragStore } from '../store/dragStore';

interface BlockPaletteProps {
  blockCount: number;
}

const ALL_BLOCKS: BlockType[] = ['LaunchApp', 'Click', 'TypeText'];

export function BlockPalette({ blockCount }: BlockPaletteProps) {
  const [search, setSearch] = useState('');

  const filtered = ALL_BLOCKS.filter(bt =>
    search === '' || BLOCK_LABELS[bt].toLowerCase().includes(search.toLowerCase())
  );

  const handleDragStart = (bt: BlockType, event: React.DragEvent) => {
    dragStore.set(bt);
    event.dataTransfer.effectAllowed = 'move';
    // Небольшой сдвиг чтобы курсор был по центру
    event.dataTransfer.setDragImage(new Image(), 0, 0);
  };

  return (
    <div className="w-[220px] bg-[#242424] border-r border-[#383838] flex flex-col select-none">
      {/* Header */}
      <div className="px-3 pt-2 pb-1">
        <h2 className="text-sm font-semibold text-gray-200 text-center">🧩 Блоки</h2>
      </div>

      {/* Search */}
      <div className="px-3 pb-2">
        <input
          type="text"
          value={search}
          onChange={e => setSearch(e.target.value)}
          placeholder="🔍 Поиск..."
          className="w-full px-2 py-1 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                     placeholder-gray-500 outline-none focus:border-[#4682b4]"
        />
      </div>

      <div className="mx-3 border-t border-[#383838]" />

      {/* Blocks */}
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-1">
        {filtered.map(bt => (
          <div
            key={bt}
            draggable
            onDragStart={(e) => handleDragStart(bt, e)}
            className="
              flex items-center gap-2 px-2 py-1.5 rounded-md cursor-grab
              bg-[#f0f0f0] border border-[#c8c8c8]
              hover:bg-[#e6e6e6] hover:border-[#a0a0a0]
              active:cursor-grabbing
            "
          >
            <span className="text-base">{BLOCK_ICONS[bt]}</span>
            <span className="text-xs text-[#282828]">{BLOCK_LABELS[bt]}</span>
          </div>
        ))}
      </div>

      {/* Stats */}
      <div className="mx-3 border-t border-[#383838] my-2" />
      <div className="px-3 pb-3">
        <div className="text-[10px] text-gray-500">Статистика:</div>
        <div className="text-[10px] text-gray-500">  Блоков: {blockCount}</div>
      </div>
    </div>
  );
}
