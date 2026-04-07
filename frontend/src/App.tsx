import React, { useState, useCallback, useMemo, useEffect, useRef } from 'react';
import { Node, Edge, OnNodesChange, OnEdgesChange, applyNodeChanges, applyEdgeChanges } from '@xyflow/react';
import { TopBar } from './components/TopBar';
import { BlockPalette } from './components/BlockPalette';
import { FlowCanvas } from './components/FlowCanvas';
import { PropertiesPanel } from './components/PropertiesPanel';
import { LogPanel } from './components/LogPanel';
import { DiagramTabBar } from './components/DiagramTabBar';
import { Project, Diagram, SerializedNode, SerializedEdge, BlockType, createProject, createDiagram } from './types';
import { executeScenario, stopExecution, type ScenarioStep, listProjects, saveProject as tauriSaveProject, loadProject as tauriLoadProject, deleteProject as tauriDeleteProject, type ProjectInfo, openProjectFile, loadProjectFromPath, saveProjectFile } from './tauri';

/** Топологическая сортировка нод по рёбрам. */
function topologicalSort(allNodes: Node[], allEdges: Edge[]): Node[] {
  const nodeMap = new Map(allNodes.map(n => [n.id, n]));
  const inDegree = new Map<string, number>();
  const adj = new Map<string, string[]>();

  for (const n of allNodes) {
    inDegree.set(n.id, 0);
    adj.set(n.id, []);
  }

  for (const e of allEdges) {
    if (nodeMap.has(e.source) && nodeMap.has(e.target)) {
      adj.get(e.source)!.push(e.target);
      inDegree.set(e.target, (inDegree.get(e.target) || 0) + 1);
    }
  }

  const queue: string[] = [];
  const roots = allNodes
    .filter(n => (inDegree.get(n.id) || 0) === 0)
    .sort((a, b) => (a.position.y || 0) - (b.position.y || 0))
    .map(n => n.id);
  queue.push(...roots);

  const result: Node[] = [];
  while (queue.length > 0) {
    const id = queue.shift()!;
    const node = nodeMap.get(id);
    if (node) result.push(node);

    const neighbors = adj.get(id) || [];
    for (const next of neighbors) {
      const deg = (inDegree.get(next) || 1) - 1;
      inDegree.set(next, deg);
      if (deg === 0) queue.push(next);
    }
  }

  const visited = new Set(result.map(n => n.id));
  const remaining = allNodes
    .filter(n => !visited.has(n.id))
    .sort((a, b) => (a.position.y || 0) - (b.position.y || 0));
  result.push(...remaining);

  return result;
}

/** Convert SerializedNode → ReactFlow Node */
function toReactFlowNode(sn: SerializedNode): Node {
  return {
    id: sn.id,
    type: 'block',
    position: sn.position,
    data: { blockType: sn.blockType as BlockType, config: sn.config },
  };
}

/** Convert ReactFlow Node → SerializedNode */
function fromReactFlowNode(n: Node): SerializedNode {
  return {
    id: n.id,
    blockType: (n.data as { blockType?: string }).blockType || 'unknown',
    position: n.position,
    config: Object.fromEntries(
      Object.entries((n.data as { config?: Record<string, unknown> }).config || {}).map(([k, v]) => [k, String(v)])
    ),
  };
}

/** Convert Edge → SerializedEdge */
function toSerializedEdge(e: Edge): SerializedEdge {
  return { id: e.id, source: e.source, target: e.target };
}

/** Convert SerializedEdge → ReactFlow Edge */
function fromSerializedEdge(se: SerializedEdge): Edge {
  return { id: se.id, source: se.source, target: se.target, animated: true, style: { stroke: '#5a8cc8', strokeWidth: 2.5 } };
}

export default function App() {
  const [project, setProject] = useState<Project>(() => createProject('Мой проект'));
  const [isRunning, setIsRunning] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [savedProjects, setSavedProjects] = useState<ProjectInfo[]>([]);

  // Nodes/edges текущей активной диаграммы
  const activeDiagram = project.diagrams.find(d => d.id === project.activeDiagramId) ?? project.diagrams[0];
  const [nodes, setNodes] = useState<Node[]>(() => activeDiagram.nodes.map(toReactFlowNode));
  const [edges, setEdges] = useState<Edge[]>(() => activeDiagram.edges.map(fromSerializedEdge));

  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  // Debounced auto-save на диск (1 секунда после последнего изменения)
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    // Очищаем предыдущий таймер
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }
    saveTimerRef.current = setTimeout(() => {
      tauriSaveProject(project as unknown as Record<string, unknown>).catch(() => { /* ignore */ });
    }, 1000);
    return () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
      }
    };
  }, [project]);

  // Загрузка списка проектов при старте
  useEffect(() => {
    listProjects().then(setSavedProjects).catch(() => { /* ignore */ });
  }, []);

  // Синхронизация nodes/edges → project state при каждом изменении
  // Не срабатывает при загрузке проекта из другой диаграммы (nodes/edges ещё не изменились)
  useEffect(() => {
    setProject(prev => ({
      ...prev,
      diagrams: prev.diagrams.map(d =>
        d.id === prev.activeDiagramId
          ? { ...d, nodes: nodes.map(fromReactFlowNode), edges: edges.map(toSerializedEdge) }
          : d
      ),
    }));
  }, [nodes, edges]);

  // Загрузка nodes/edges при смене активной диаграммы
  useEffect(() => {
    const target = project.diagrams.find(d => d.id === project.activeDiagramId);
    if (target) {
      setNodes(target.nodes.map(toReactFlowNode));
      setEdges(target.edges.map(fromSerializedEdge));
      setSelectedNodeId(null);
    }
  }, [project.activeDiagramId]);

  const onNodesChange: OnNodesChange = useCallback(
    (changes) => setNodes((nds) => applyNodeChanges(changes, nds)),
    []
  );
  const onEdgesChange: OnEdgesChange = useCallback(
    (changes) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  );

  const handleUpdateNode = useCallback((nodeId: string, updates: { position?: { x: number; y: number }; data?: Record<string, unknown> }) => {
    setNodes((nds) => nds.map((n) => (n.id !== nodeId ? n : { ...n, ...updates })));
  }, []);

  const handleDeleteNode = useCallback((nodeId: string) => {
    setNodes((nds) => nds.filter((n) => n.id !== nodeId));
    setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    setSelectedNodeId(null);
  }, []);

  const handleDuplicateNode = useCallback((nodeId: string) => {
    setNodes((nds) => {
      const src = nds.find((n) => n.id === nodeId);
      if (!src) return nds;
      return [...nds, { ...src, id: crypto.randomUUID(), position: { x: src.position.x + 30, y: src.position.y + 30 }, data: { ...src.data } }];
    });
  }, []);

  const handleSelectDiagram = useCallback((id: string) => {
    setProject(prev => ({ ...prev, activeDiagramId: id }));
  }, []);

  const handleAddDiagram = useCallback(() => {
    setProject(prev => {
      const num = prev.diagrams.length + 1;
      const newDiagram = createDiagram(`Diagram${num}`);
      const updated = { ...prev, diagrams: [...prev.diagrams, newDiagram], activeDiagramId: newDiagram.id };
      setNodes(newDiagram.nodes.map(toReactFlowNode));
      setEdges(newDiagram.edges.map(fromSerializedEdge));
      setSelectedNodeId(null);
      return updated;
    });
  }, []);

  const handleNewProject = useCallback(() => {
    const name = prompt('Имя нового проекта:', 'Новый проект');
    if (name) {
      const p = createProject(name);
      setProject(p);
      setNodes(p.diagrams[0].nodes.map(toReactFlowNode));
      setEdges(p.diagrams[0].edges.map(fromSerializedEdge));
      setSelectedNodeId(null);
      setLogs([]);
      listProjects().then(setSavedProjects);
    }
  }, []);

  const handleOpenFile = useCallback(async () => {
    const filePath = await openProjectFile();
    if (!filePath) return;
    try {
      const data = await loadProjectFromPath(filePath);
      const proj = data as unknown as Project;
      setProject(proj);
      const active = proj.diagrams.find(d => d.id === proj.activeDiagramId) || proj.diagrams[0];
      setNodes(active.nodes.map(toReactFlowNode));
      setEdges(active.edges.map(fromSerializedEdge));
      setSelectedNodeId(null);
      setLogs((prev) => [...prev, `📂 Проект "${proj.name}" загружен`]);
      listProjects().then(setSavedProjects);
    } catch (err) {
      setLogs((prev) => [...prev, `❌ Не удалось прочитать файл: ${err}`]);
    }
  }, []);

  const handleLoadProject = useCallback(async (fileInfo: ProjectInfo) => {
    try {
      const data = await tauriLoadProject(fileInfo.file_name);
      const proj = data as unknown as Project;
      setProject(proj);
      const active = proj.diagrams.find(d => d.id === proj.activeDiagramId) || proj.diagrams[0];
      setNodes(active.nodes.map(toReactFlowNode));
      setEdges(active.edges.map(fromSerializedEdge));
      setSelectedNodeId(null);
      setLogs((prev) => [...prev, `📂 Проект "${proj.name}" загружен`]);
    } catch {
      setLogs((prev) => [...prev, '❌ Ошибка загрузки проекта']);
    }
  }, []);

  const handleDeleteProject = useCallback(async (fileInfo: ProjectInfo) => {
    if (!confirm(`Удалить проект "${fileInfo.name}"?`)) return;
    try {
      await tauriDeleteProject(fileInfo.file_name);
      listProjects().then(setSavedProjects);
      setLogs((prev) => [...prev, `🗑 Проект "${fileInfo.name}" удалён`]);
    } catch {
      setLogs((prev) => [...prev, '❌ Ошибка удаления проекта']);
    }
  }, []);

  const handleSaveProject = useCallback(async () => {
    const data = {
      ...project,
      diagrams: project.diagrams.map(d =>
        d.id === project.activeDiagramId
          ? { ...d, nodes: nodes.map(fromReactFlowNode), edges: edges.map(toSerializedEdge) }
          : d
      ),
    };
    const path = await saveProjectFile(data as unknown as Record<string, unknown>, project.name);
    if (path) {
      setLogs((prev) => [...prev, `💾 Проект сохранён: ${path}`]);
      listProjects().then(setSavedProjects);
    }
  }, [project, nodes, edges]);

  const handleRun = useCallback(async () => {
    if (nodes.length === 0) {
      setLogs((prev) => [...prev, '⚠ Нет блоков для выполнения']);
      return;
    }

    const hasStart = nodes.some((n) => (n.data as { blockType?: string }).blockType === 'Start');
    if (!hasStart) {
      setLogs((prev) => [...prev, '❌ Ошибка: на диаграмме отсутствует блок «Старт». Добавьте блок «Старт» как точку входа.']);
      return;
    }

    setIsRunning(true);
    setLogs([]);

    const sorted = topologicalSort(nodes, edges);
    const steps: ScenarioStep[] = sorted.map((n) => ({
      type: (n.data as { blockType?: string }).blockType || 'unknown',
      config: Object.fromEntries(
        Object.entries((n.data as { config?: Record<string, unknown> }).config || {}).map(([k, v]) => [k, String(v)])
      ),
    }));

    try {
      const result = await executeScenario(steps);
      setLogs(result.log);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setLogs((prev) => [...prev, `❌ Ошибка: ${msg}`]);
    } finally {
      setIsRunning(false);
    }
  }, [nodes, edges]);

  const handleStop = useCallback(async () => {
    try { await stopExecution(); } catch { /* ignore */ }
    setIsRunning(false);
    setLogs((prev) => [...prev, '⏹ Сценарий остановлен']);
  }, []);

  const handleSave = useCallback(() => { handleSaveProject(); }, [handleSaveProject]);

  const selectedNode = useMemo(() => nodes.find((n) => n.id === selectedNodeId) || null, [nodes, selectedNodeId]);

  return (
    <div className="flex flex-col h-screen bg-white">
      <TopBar blockCount={nodes.length} isRunning={isRunning} onRun={handleRun} onStop={handleStop} onSave={handleSave} />
      <DiagramTabBar
        project={project}
        activeDiagramId={project.activeDiagramId}
        savedProjects={savedProjects}
        onSelectDiagram={handleSelectDiagram}
        onAddDiagram={handleAddDiagram}
        onOpenFile={handleOpenFile}
        onLoadProject={handleLoadProject}
        onDeleteProject={handleDeleteProject}
        onNewProject={handleNewProject}
        onSaveProject={handleSaveProject}
      />
      <div className="flex flex-1 overflow-hidden">
        <BlockPalette blockCount={nodes.length} />
        <div className="flex-1 flex flex-col">
          <div className="flex-1">
            <FlowCanvas
              nodes={nodes} edges={edges}
              onNodesChange={onNodesChange} onEdgesChange={onEdgesChange}
              selectedNodeId={selectedNodeId} setSelectedNodeId={setSelectedNodeId}
              onSetNodes={(fn) => setNodes(fn)} onSetEdges={(fn) => setEdges(fn)}
            />
          </div>
          <LogPanel logs={logs} />
        </div>
        <PropertiesPanel
          selectedNode={selectedNode}
          onUpdateNode={handleUpdateNode}
          onDeleteNode={handleDeleteNode}
          onDuplicateNode={handleDuplicateNode}
        />
      </div>
    </div>
  );
}
