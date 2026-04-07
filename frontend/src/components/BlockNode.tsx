import { memo } from 'react';
import { Handle, Position } from '@xyflow/react';
import { BlockType, BLOCK_LABELS, BLOCK_ICONS, BLOCK_ACCENT } from '../types';

interface BlockNodeProps {
  data: {
    blockType: BlockType;
    config: Record<string, string>;
  };
  selected: boolean;
  id: string;
}

export const BlockNode = memo(({ data, selected }: BlockNodeProps) => {
  const blockType = data.blockType;
  const config = data.config || {};
  const label = BLOCK_LABELS[blockType] || blockType;
  const icon = BLOCK_ICONS[blockType] || '📦';
  const accent = BLOCK_ACCENT[blockType] || '#828282';

  const firstValue = Object.values(config)[0] || '';
  const display = firstValue.length > 35 ? firstValue.slice(0, 35) + '...' : firstValue;

  return (
    <div
      className={`
        relative w-[220px] bg-white rounded-md shadow-sm transition-shadow
        ${selected ? 'outline outline-2 outline-offset-1' : 'ring-1 ring-gray-200'}
      `}
      style={selected ? { outlineColor: accent } : {}}
    >
      {/* Accent stripe */}
      <div
        className="absolute left-0 top-0 bottom-0 w-1 rounded-l-md"
        style={{ backgroundColor: accent }}
      />

      {/* Input handle — у Start нет входной точки */}
      {blockType !== 'Start' && (
        <Handle
          type="target"
          position={Position.Top}
          className="!w-3 !h-3 !bg-blue-500 !border-2 !border-white !-top-1.5"
          id="input"
        />
      )}

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Bottom}
        className="!w-3 !h-3 !bg-blue-500 !border-2 !border-white !-bottom-1.5"
        id="output"
      />

      {/* Content */}
      <div className="pl-4 pr-3 py-2.5">
        <div className="flex items-center gap-1.5">
          <span className="text-base">{icon}</span>
          <span className="text-sm font-medium text-gray-800">{label}</span>
        </div>
        {display && (
          <div className="text-[10px] text-gray-400 mt-1 truncate">{display}</div>
        )}
      </div>
    </div>
  );
});

BlockNode.displayName = 'BlockNode';
