import { useState } from 'react';
import { Project } from '../types';
import { ProjectInfo } from '../tauri';

interface DiagramTabBarProps {
  project: Project;
  activeDiagramId: string;
  savedProjects: ProjectInfo[];
  onSelectDiagram: (id: string) => void;
  onAddDiagram: () => void;
  onOpenFile: () => void;
  onLoadProject: (info: ProjectInfo) => void;
  onDeleteProject: (info: ProjectInfo) => void;
  onNewProject: () => void;
  onSaveProject: () => void;
}

export function DiagramTabBar({
  project,
  activeDiagramId,
  savedProjects,
  onSelectDiagram,
  onAddDiagram,
  onOpenFile,
  onLoadProject,
  onDeleteProject,
  onNewProject,
  onSaveProject,
}: DiagramTabBarProps) {
  const [showProjects, setShowProjects] = useState(false);

  return (
    <div className="flex items-center bg-[#1e1e1e] border-b border-[#383838] px-2 h-8 shrink-0">
      {/* Project name */}
      <div className="px-2 py-0.5 text-xs bg-[#303030] text-gray-200 rounded border border-[#383838] mr-2 w-32 truncate" title={project.name}>
        📁 {project.name}
      </div>

      {/* Diagram tabs */}
      <div className="flex items-center gap-0.5 overflow-x-auto flex-1">
        {project.diagrams.map(d => (
          <button
            key={d.id}
            onClick={() => onSelectDiagram(d.id)}
            className={`px-3 py-1 text-xs rounded-t-md border-b-2 transition-colors whitespace-nowrap ${
              d.id === activeDiagramId
                ? 'bg-[#2a2a2a] text-gray-200 border-[#4682b4]'
                : 'bg-[#1e1e1e] text-gray-500 border-transparent hover:text-gray-300 hover:bg-[#252525]'
            }`}
          >
            {d.name === 'Main' ? '🏠 ' : '📄 '}{d.name}
          </button>
        ))}
        <button
          onClick={onAddDiagram}
          className="px-2 py-1 text-xs text-gray-500 hover:text-gray-300 hover:bg-[#252525] rounded transition-colors ml-1"
          title="Добавить диаграмму"
        >
          +
        </button>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1 ml-2 relative">
        <button onClick={onNewProject} className="px-2 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-[#252525] rounded transition-colors" title="Новый проект">📁</button>
        <button
          onClick={onOpenFile}
          className="px-2 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-[#252525] rounded transition-colors"
          title="Открыть файл проекта"
        >
          📂
        </button>
        <button
          onClick={() => setShowProjects(!showProjects)}
          className="px-2 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-[#252525] rounded transition-colors"
          title="Сохранённые проекты"
        >
          📋
        </button>
        <button onClick={onSaveProject} className="px-2 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-[#252525] rounded transition-colors" title="Сохранить проект">💾</button>

        {/* Projects dropdown */}
        {showProjects && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setShowProjects(false)} />
            <div className="absolute right-0 top-8 z-50 w-64 bg-[#2a2a2a] border border-[#383838] rounded-md shadow-xl overflow-y-auto max-h-80">
              <div className="px-3 py-2 text-xs font-semibold text-gray-300 border-b border-[#383838]">
                Сохранённые проекты
              </div>
              {savedProjects.length === 0 ? (
                <div className="px-3 py-3 text-xs text-gray-500 text-center">Нет сохранённых проектов</div>
              ) : (
                savedProjects.map(p => (
                  <div key={p.file_name} className="flex items-center px-3 py-1.5 hover:bg-[#333] group">
                    <button
                      onClick={() => { onLoadProject(p); setShowProjects(false); }}
                      className="flex-1 text-left text-xs text-gray-300 truncate"
                      title={`Загрузить "${p.name}"`}
                    >
                      📄 {p.name}
                    </button>
                    <button
                      onClick={() => onDeleteProject(p)}
                      className="ml-1 text-xs text-gray-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                      title="Удалить"
                    >
                      ✕
                    </button>
                  </div>
                ))
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
