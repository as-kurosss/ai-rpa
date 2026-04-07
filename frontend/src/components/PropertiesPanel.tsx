import { Node } from '@xyflow/react';
import { BlockType, BLOCK_LABELS, BLOCK_ICONS } from '../types';

const FIELD_LABELS: Record<string, string> = {
  pid: '🎯 PID (число или имя переменной)',
  selector: 'Селектор',
  target_selector: 'Селектор цели',
  text: 'Текст',
  app: 'Приложение',
  var_name: 'Имя переменной',
  process_name: 'Имя процесса',
  force: 'Принудительно',
  keys: 'Клавиши',
  delay_ms: 'Задержка (мс)',
  timeout_ms: 'Таймаут (мс)',
  interval_ms: 'Интервал (мс)',
  max_attempts: 'Попытки',
  duration_ms: 'Длительность (мс)',
  file_path: 'Путь к файлу',
  content: 'Содержимое',
  append: 'Добавлять',
  output_path: 'Путь сохранения',
  diagram_id: 'ID диаграммы',
};

interface PropertiesPanelProps {
  selectedNode: Node | null;
  onUpdateNode: (nodeId: string, updates: { position?: { x: number; y: number }; data?: Record<string, unknown> }) => void;
  onDeleteNode: (nodeId: string) => void;
  onDuplicateNode: (nodeId: string) => void;
}

export function PropertiesPanel({
  selectedNode,
  onUpdateNode,
  onDeleteNode,
  onDuplicateNode,
}: PropertiesPanelProps) {
  if (!selectedNode) {
    return (
      <div className="w-[260px] bg-[#242424] border-l border-[#383838] flex flex-col select-none">
        <div className="px-3 pt-2 pb-1">
          <h2 className="text-sm font-semibold text-gray-200 text-center">⚙ Свойства</h2>
        </div>
        <div className="flex-1 flex items-center justify-center px-3">
          <p className="text-xs text-gray-500 text-center">
            Выберите блок на canvas<br />для редактирования
          </p>
        </div>
      </div>
    );
  }

  const blockType = (selectedNode.data as { blockType?: BlockType }).blockType || 'Click';
  const config = (selectedNode.data as { config?: Record<string, string> }).config || {};

  const handleConfigChange = (key: string, value: string) => {
    onUpdateNode(selectedNode.id, {
      data: {
        ...selectedNode.data,
        config: { ...config, [key]: value },
      },
    });
  };

  return (
    <div className="w-[260px] bg-[#242424] border-l border-[#383838] flex flex-col select-none overflow-y-auto">
      {/* Header */}
      <div className="px-3 pt-2 pb-1">
        <h2 className="text-sm font-semibold text-gray-200 text-center">⚙ Свойства</h2>
      </div>

      <div className="px-3 pb-3 space-y-3">
        {/* Block info */}
        <div className="flex items-center gap-2">
          <span className="text-xl">{BLOCK_ICONS[blockType]}</span>
          <div>
            <div className="text-sm font-semibold text-gray-200">{BLOCK_LABELS[blockType]}</div>
            <div className="text-[10px] text-gray-500">ID: {selectedNode.id.slice(0, 8)}</div>
          </div>
        </div>

        {/* Position */}
        <div>
          <div className="text-[10px] text-gray-500 mb-1">Позиция:</div>
          <div className="flex items-center gap-2">
            <label className="text-xs text-gray-400">X:</label>
            <input
              type="number"
              defaultValue={Math.round(selectedNode.position.x)}
              onBlur={e => onUpdateNode(selectedNode.id, {
                position: { ...selectedNode.position, x: Number(e.target.value) }
              })}
              className="w-20 px-1 py-0.5 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                         outline-none focus:border-[#4682b4]"
            />
            <label className="text-xs text-gray-400">Y:</label>
            <input
              type="number"
              defaultValue={Math.round(selectedNode.position.y)}
              onBlur={e => onUpdateNode(selectedNode.id, {
                position: { ...selectedNode.position, y: Number(e.target.value) }
              })}
              className="w-20 px-1 py-0.5 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                         outline-none focus:border-[#4682b4]"
            />
          </div>
        </div>

        {/* Config */}
        {Object.entries(config).map(([key, value]) => {
          const label = FIELD_LABELS[key] || key;
          const isPidField = key === 'pid';
          const isFilePath = key === 'file_path';
          const isContent = key === 'content';
          const isTextarea = key === 'text' || key === 'content';
          return (
            <div key={key}>
              <div className="text-[10px] text-gray-500 mb-1">{label}:</div>
              {isPidField && (
                <div className="text-[9px] text-gray-600 mb-0.5">Число (напр: 12345) или имя переменной (напр: my_pid)</div>
              )}
              {isFilePath && (
                <div className="text-[9px] text-gray-600 mb-0.5">Текст в кавычках = путь, без кавычек = переменная</div>
              )}
              {isContent && (
                <div className="text-[9px] text-gray-600 mb-0.5">Имя переменной = её значение, текст = как есть</div>
              )}
              {isTextarea ? (
                <textarea
                  value={value}
                  onChange={e => handleConfigChange(key, e.target.value)}
                  rows={3}
                  placeholder={isContent ? 'extracted_text  или  "literal text"' : ''}
                  className="w-full px-2 py-1 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                             outline-none focus:border-[#4682b4] resize-none"
                />
              ) : (
                <input
                  type="text"
                  value={value}
                  onChange={e => handleConfigChange(key, e.target.value)}
                  placeholder={isPidField ? '12345 или my_pid' : isFilePath ? '"C:\\file.txt" или my_path' : ''}
                  className="w-full px-2 py-1 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                             outline-none focus:border-[#4682b4]"
                />
              )}
            </div>
          );
        })}

        {/* Actions */}
        <div className="flex gap-2 pt-2">
          <button
            onClick={() => onDeleteNode(selectedNode.id)}
            className="flex-1 px-2 py-1 text-xs bg-red-900/30 text-red-400 rounded border border-red-900/50
                       hover:bg-red-900/50 transition-colors"
          >
            🗑 Удалить
          </button>
          <button
            onClick={() => onDuplicateNode(selectedNode.id)}
            className="flex-1 px-2 py-1 text-xs bg-blue-900/30 text-blue-400 rounded border border-blue-900/50
                       hover:bg-blue-900/50 transition-colors"
          >
            📋 Дублировать
          </button>
        </div>
      </div>
    </div>
  );
}
