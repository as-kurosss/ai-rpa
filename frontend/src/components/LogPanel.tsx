import { useRef, useEffect } from 'react';

interface LogPanelProps {
  logs: string[];
}

export function LogPanel({ logs }: LogPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const getLogColor = (entry: string) => {
    if (entry.startsWith('❌') || entry.startsWith('⏹')) return 'text-red-400';
    if (entry.startsWith('✓')) return 'text-green-400';
    if (entry.startsWith('▶')) return 'text-blue-400';
    if (entry.startsWith('🔗')) return 'text-purple-400';
    if (entry.startsWith('+')) return 'text-gray-300';
    return 'text-gray-500';
  };

  return (
    <div className="h-40 bg-[#1e1e1e] border-t border-[#383838] flex flex-col shrink-0">
      <div className="px-3 py-1 border-b border-[#383838]">
        <span className="text-[10px] text-gray-500">📋 Лог:</span>
      </div>
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-3 py-1 space-y-0.5 font-mono"
      >
        {logs.length === 0 ? (
          <div className="text-[10px] text-gray-600">Нет записей</div>
        ) : (
          logs.map((entry, i) => (
            <div key={i} className={`text-[10px] ${getLogColor(entry)}`}>
              {entry}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
