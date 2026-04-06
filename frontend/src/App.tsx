import { useState, useCallback, useMemo } from 'react';
import { Node, Edge, OnNodesChange, OnEdgesChange, applyNodeChanges, applyEdgeChanges } from '@xyflow/react';
import { TopBar } from './components/TopBar';
import { BlockPalette } from './components/BlockPalette';
import { FlowCanvas } from './components/FlowCanvas';
import { PropertiesPanel } from './components/PropertiesPanel';
import { LogPanel } from './components/LogPanel';
import { BlockType, createDefaultConfig } from './types';
import { dragStore } from './store/dragStore';

// Начальные демо-блоки
function createInitialNodes(): Node[] {
  return [
    {
      id: '1',
      type: 'block',
      position: { x: 100, y: 80 },
      data: { blockType: 'LaunchApp' as BlockType, config: { app: 'notepad' } },
    },
    {
      id: '2',
      type: 'block',
      position: { x: 100, y: 220 },
      data: { blockType: 'Click' as BlockType, config: { selector: 'classname=Edit' } },
    },
    {
      id: '3',
      type: 'block',
      position: { x: 100, y: 360 },
      data: { blockType: 'TypeText' as BlockType, config: { selector: 'classname=Edit', text: 'Привет из RPA Studio!' } },
    },
  ];
}

function createInitialEdges(): Edge[] {
  return [
    { id: 'e1-2', source: '1', target: '2', animated: true, style: { stroke: '#5a8cc8', strokeWidth: 2.5 } },
    { id: 'e2-3', source: '2', target: '3', animated: true, style: { stroke: '#5a8cc8', strokeWidth: 2.5 } },
  ];
}

export default function App() {
  const [nodes, setNodes] = useState<Node[]>(createInitialNodes);
  const [edges, setEdges] = useState<Edge[]>(createInitialEdges);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);

  const onNodesChange: OnNodesChange = useCallback(
    (changes) => setNodes((nds) => applyNodeChanges(changes, nds)),
    []
  );
  const onEdgesChange: OnEdgesChange = useCallback(
    (changes) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  );

  const handleDragStart = useCallback((blockType: BlockType, _event: React.DragEvent) => {
    dragStore.set(blockType);
  }, []);

  const handleUpdateNode = useCallback((nodeId: string, updates: { position?: { x: number; y: number }; data?: Record<string, unknown> }) => {
    setNodes((nds) =>
      nds.map((n) => {
        if (n.id !== nodeId) return n;
        return { ...n, ...updates };
      })
    );
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
      return [
        ...nds,
        { ...src, id: crypto.randomUUID(), position: { x: src.position.x + 30, y: src.position.y + 30 }, data: { ...src.data } },
      ];
    });
  }, []);

  const handleRun = useCallback(() => {
    setIsRunning(true);
    setLogs((prev) => [...prev, '▶ Запуск сценария...']);

    const steps = nodes.map((n) => ({
      type: (n.data as { blockType?: string }).blockType || 'unknown',
      config: (n.data as { config?: Record<string, string> }).config || {},
    }));
    setLogs((prev) => [...prev, `  Блоков: ${steps.length}`]);
    steps.forEach((s, i) => {
      setLogs((prev) => [...prev, `  [${i + 1}] ${s.type}: ${JSON.stringify(s.config)}`]);
    });
    setLogs((prev) => [...prev, '✓ Сценарий завершён']);
    setIsRunning(false);
  }, [nodes]);

  const handleStop = useCallback(() => {
    setIsRunning(false);
    setLogs((prev) => [...prev, '⏹ Сценарий остановлен']);
  }, []);

  const handleSave = useCallback(() => {
    const data = {
      nodes: nodes.map((n) => ({
        id: n.id,
        blockType: (n.data as { blockType?: string }).blockType,
        position: n.position,
        config: (n.data as { config?: Record<string, string> }).config,
      })),
      edges: edges.map((e) => ({ from: e.source, to: e.target })),
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'scenario.json';
    a.click();
    URL.revokeObjectURL(url);
    setLogs((prev) => [...prev, '💾 Сохранено']);
  }, [nodes, edges]);

  const selectedNode = useMemo(
    () => nodes.find((n) => n.id === selectedNodeId) || null,
    [nodes, selectedNodeId]
  );

  return (
    <div className="flex flex-col h-screen bg-white">
      <TopBar
        blockCount={nodes.length}
        isRunning={isRunning}
        onRun={handleRun}
        onStop={handleStop}
        onSave={handleSave}
      />
      <div className="flex flex-1 overflow-hidden">
        <BlockPalette blockCount={nodes.length} />
        <div className="flex-1 flex flex-col">
          <div className="flex-1">
            <FlowCanvas
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              selectedNodeId={selectedNodeId}
              setSelectedNodeId={setSelectedNodeId}
              onSetNodes={(fn) => setNodes(fn)}
              onSetEdges={(fn) => setEdges(fn)}
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
