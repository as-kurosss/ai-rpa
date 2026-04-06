interface TopBarProps {
  blockCount: number;
  isRunning: boolean;
  onRun: () => void;
  onStop: () => void;
  onSave: () => void;
}

export function TopBar({ blockCount, isRunning, onRun, onStop, onSave }: TopBarProps) {
  return (
    <div className="h-10 bg-[#242424] border-b border-[#383838] flex items-center px-4 gap-3 select-none shrink-0">
      <h1 className="text-sm font-semibold text-gray-200 mr-4">🤖 RPA Studio</h1>

      <button
        onClick={isRunning ? onStop : onRun}
        disabled={blockCount === 0}
        className={`
          px-3 py-1 text-xs rounded font-medium transition-colors
          ${isRunning
            ? 'bg-red-800 text-red-100 hover:bg-red-700'
            : 'bg-green-800 text-green-100 hover:bg-green-700 disabled:opacity-40 disabled:cursor-not-allowed'
          }
        `}
      >
        {isRunning ? '⏹ Стоп' : '▶ Запуск'}
      </button>

      <button
        onClick={onSave}
        className="px-3 py-1 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838]
                   hover:bg-[#404040] transition-colors"
      >
        💾 Сохранить
      </button>

      <div className="flex-1" />

      <span className="text-[10px] text-gray-500">Блоков: {blockCount}</span>
    </div>
  );
}
