import { useCallback, useEffect, useRef, useState } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  addEdge,
  OnConnect,
  Node,
  Edge,
  BackgroundVariant,
  ReactFlowProvider,
  useReactFlow,
  OnNodesChange,
  OnEdgesChange,
  BaseEdge,
  EdgeProps,
  getSmoothStepPath,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { BlockNode } from './BlockNode';
import { BlockType, createDefaultConfig, BLOCK_LABELS, BLOCK_ICONS } from '../types';
import { dragState } from './BlockPalette';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: Record<string, any> = { block: BlockNode };

const VALID_BLOCKS: BlockType[] = [
  'Start',
  'LaunchApp', 'CloseApp',
  'Click', 'DoubleClick', 'RightClick', 'MoveMouse', 'DragAndDrop',
  'TypeText', 'KeyPress',
  'ExtractText', 'Screenshot',
  'Wait', 'WaitForElement', 'Retry', 'Condition',
  'ReadFile', 'WriteFile',
];

interface FlowCanvasContentProps {
  nodes: Node[];
  edges: Edge[];
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  selectedNodeId: string | null;
  setSelectedNodeId: (id: string | null) => void;
  onSetNodes: (fn: (nodes: Node[]) => Node[]) => void;
  onSetEdges: (fn: (edges: Edge[]) => Edge[]) => void;
}

const SNAP_DISTANCE = 100;
const VERTICAL_GAP = 60;
const BLOCK_HEIGHT = 65;
const BLOCK_WIDTH = 220;
const SNAP_EDGE_ID = '__snap_edge__';

function findNearestNodeAbove(
  nodes: Node[],
  position: { x: number; y: number },
  excludeId: string
): Node | null {
  let nearest: Node | null = null;
  let minDist = SNAP_DISTANCE;
  for (const node of nodes) {
    if (node.id === excludeId) continue;
    if (node.type !== 'block') continue;
    const nodeBottom = node.position.y + BLOCK_HEIGHT;
    const dy = position.y - nodeBottom;
    const dx = Math.abs(position.x - node.position.x);
    if (dy >= 0 && dy < SNAP_DISTANCE && dx < SNAP_DISTANCE * 1.5) {
      if (dy < minDist) { minDist = dy; nearest = node; }
    }
  }
  return nearest;
}

function hasEdgeBetween(edges: Edge[], sourceId: string, targetId: string): boolean {
  return edges.some(e => e.id !== SNAP_EDGE_ID && e.source === sourceId && e.target === targetId);
}

function SnapEdge({ sourceX, sourceY, targetX, targetY }: EdgeProps) {
  const [edgePath] = getSmoothStepPath({ sourceX, sourceY, targetX, targetY });
  return (
    <BaseEdge path={edgePath} style={{
      stroke: '#5a8cc8', strokeWidth: 2, strokeDasharray: '6,4', opacity: 0.7,
    }} />
  );
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const edgeTypes: Record<string, any> = { snap: SnapEdge };

function FlowCanvasContent({
  nodes, edges, onNodesChange, onEdgesChange,
  selectedNodeId, setSelectedNodeId, onSetNodes, onSetEdges,
}: FlowCanvasContentProps) {
  const { screenToFlowPosition } = useReactFlow();
  const containerRef = useRef<HTMLDivElement>(null);
  const handlersRef = useRef({ onSetNodes, onSetEdges, screenToFlowPosition });
  const edgesRef = useRef(edges);
  const nodesRef = useRef(nodes);
  const [isOver, setIsOver] = useState(false);
  const [ghostPos, setGhostPos] = useState<{ x: number; y: number } | null>(null);
  const [isSnapping, setIsSnapping] = useState(false);

  const dragStateRef = useRef<{ draggedNodeId: string | null; snapTargetId: string | null }>(
    { draggedNodeId: null, snapTargetId: null }
  );
  const paletteSnapRef = useRef<{ snapTargetId: string | null }>({ snapTargetId: null });

  useEffect(() => {
    handlersRef.current = { onSetNodes, onSetEdges, screenToFlowPosition };
  }, [onSetNodes, onSetEdges, screenToFlowPosition]);

  useEffect(() => { nodesRef.current = nodes; }, [nodes]);
  useEffect(() => { edgesRef.current = edges; }, [edges]);

  const onConnect: OnConnect = useCallback((connection) => {
    onSetEdges((eds) => addEdge({ ...connection, animated: false, style: { stroke: '#5a8cc8', strokeWidth: 2.5 } }, eds));
  }, [onSetEdges]);

  const onNodeClick = useCallback((_event: React.MouseEvent, node: Node) => setSelectedNodeId(node.id), [setSelectedNodeId]);
  const onPaneClick = useCallback(() => setSelectedNodeId(null), [setSelectedNodeId]);

  const clearSnapEdge = useCallback(() => {
    onSetEdges((eds) => eds.filter(e => e.id !== SNAP_EDGE_ID));
  }, [onSetEdges]);

  // --- Drag existing nodes ---
  const onNodeDrag = useCallback((_event: React.MouseEvent, node: Node) => {
    const allNodes = nodesRef.current;
    const allEdges = edgesRef.current;
    const snapTarget = findNearestNodeAbove(allNodes, node.position, node.id);
    if (snapTarget) {
      if (hasEdgeBetween(allEdges, snapTarget.id, node.id)) {
        clearSnapEdge();
        dragStateRef.current = { draggedNodeId: null, snapTargetId: null };
        return;
      }
      onSetEdges((eds) => {
        const withoutSnap = eds.filter(e => e.id !== SNAP_EDGE_ID);
        return [...withoutSnap, {
          id: SNAP_EDGE_ID, source: snapTarget.id, target: node.id,
          sourceHandle: 'output', targetHandle: 'input', type: 'snap',
        }];
      });
      dragStateRef.current = { draggedNodeId: node.id, snapTargetId: snapTarget.id };
    } else {
      clearSnapEdge();
      dragStateRef.current = { draggedNodeId: null, snapTargetId: null };
    }
  }, [clearSnapEdge, onSetEdges]);

  const onNodeDragStop = useCallback((_event: React.MouseEvent, _node: Node, _allNodes: Node[]) => {
    const { draggedNodeId, snapTargetId } = dragStateRef.current;
    clearSnapEdge();
    if (draggedNodeId && snapTargetId) {
      const currentNodes = nodesRef.current;
      const draggedNode = currentNodes.find(n => n.id === draggedNodeId);
      const snapTarget = currentNodes.find(n => n.id === snapTargetId);
      if (draggedNode && snapTarget) {
        const snapPosition = { x: snapTarget.position.x, y: snapTarget.position.y + BLOCK_HEIGHT + VERTICAL_GAP };
        onSetNodes((nds) => nds.map(n => n.id === draggedNodeId ? { ...n, position: snapPosition } : n));
        if (!hasEdgeBetween(edgesRef.current, snapTargetId, draggedNodeId)) {
          onSetEdges((eds) => {
            const withoutSnap = eds.filter(e => e.id !== SNAP_EDGE_ID);
            return [...withoutSnap, {
              id: `e-${snapTargetId}-${draggedNodeId}`, source: snapTargetId, target: draggedNodeId,
              sourceHandle: 'output', targetHandle: 'input', animated: false,
              style: { stroke: '#5a8cc8', strokeWidth: 2.5 }, type: 'smoothstep',
            }];
          });
        }
      }
    }
    dragStateRef.current = { draggedNodeId: null, snapTargetId: null };
  }, [clearSnapEdge, onSetEdges, onSetNodes]);

  // --- Drag from palette ---
  useEffect(() => {
    let dragging = false;

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const over = e.clientX >= rect.left && e.clientX <= rect.right &&
        e.clientY >= rect.top && e.clientY <= rect.bottom;
      setIsOver(over);

      if (over) {
        setGhostPos({ x: e.clientX - rect.left, y: e.clientY - rect.top });
        const flowPos = handlersRef.current.screenToFlowPosition({ x: e.clientX, y: e.clientY });
        const snapTarget = findNearestNodeAbove(nodesRef.current, flowPos, '');

        if (snapTarget) {
          setIsSnapping(true);
          paletteSnapRef.current = { snapTargetId: snapTarget.id };
        } else {
          setIsSnapping(false);
          paletteSnapRef.current = { snapTargetId: null };
        }
      } else {
        setGhostPos(null);
        setIsSnapping(false);
        paletteSnapRef.current = { snapTargetId: null };
      }
    };

    const handleMouseUp = (e: MouseEvent) => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      setIsSnapping(false);
      const savedSnapTargetId = paletteSnapRef.current.snapTargetId;
      paletteSnapRef.current = { snapTargetId: null };

      if (!dragging || !dragState.blockType) {
        dragState.set(null); setGhostPos(null); setIsOver(false); dragging = false; return;
      }
      if (!containerRef.current) {
        dragState.set(null); setGhostPos(null); setIsOver(false); dragging = false; return;
      }

      const rect = containerRef.current.getBoundingClientRect();
      const over = e.clientX >= rect.left && e.clientX <= rect.right &&
        e.clientY >= rect.top && e.clientY <= rect.bottom;

      if (over && dragState.blockType && VALID_BLOCKS.includes(dragState.blockType)) {
        const blockType = dragState.blockType as BlockType;
        const position = handlersRef.current.screenToFlowPosition({ x: e.clientX, y: e.clientY });

        let snapTarget = findNearestNodeAbove(nodesRef.current, position, '');
        if (!snapTarget && savedSnapTargetId) {
          snapTarget = nodesRef.current.find(n => n.id === savedSnapTargetId) || null;
        }

        let finalPosition = position;
        if (snapTarget) {
          finalPosition = { x: snapTarget.position.x, y: snapTarget.position.y + BLOCK_HEIGHT + VERTICAL_GAP };
        }

        const newNodeId = crypto.randomUUID();
        const newNode: Node = {
          id: newNodeId, type: 'block', position: finalPosition,
          data: { blockType, config: createDefaultConfig(blockType) },
        };

        handlersRef.current.onSetNodes((nds) => [...nds, newNode]);

        if (snapTarget) {
          handlersRef.current.onSetEdges((eds) => [...eds, {
            id: `e-${snapTarget.id}-${newNodeId}`, source: snapTarget.id, target: newNodeId,
            sourceHandle: 'output', targetHandle: 'input', animated: false,
            style: { stroke: '#5a8cc8', strokeWidth: 2.5 }, type: 'smoothstep',
          }]);
        }
      }

      dragState.set(null); dragging = false; setGhostPos(null); setIsOver(false);
    };

    const startListening = () => {
      if (dragging) return;
      dragging = true;
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('rpa-drag-start', startListening);
    return () => {
      window.removeEventListener('rpa-drag-start', startListening);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  return (
    <div className="w-full h-full relative" ref={containerRef}>
      {ghostPos && dragState.blockType && (
        <div className={`absolute pointer-events-none z-50 opacity-80 bg-white rounded-lg shadow-lg px-3 py-2 text-sm font-medium transition-colors ${isSnapping ? 'border-2 border-green-400' : 'border-2 border-blue-400'}`}
          style={{ left: ghostPos.x - 50, top: ghostPos.y - 15 }}>
          {BLOCK_ICONS[dragState.blockType]} {BLOCK_LABELS[dragState.blockType]}
          {isSnapping && <span className="ml-1">🔗</span>}
        </div>
      )}
      {isOver && dragState.isDragging && (
        <div className="absolute inset-0 z-40 pointer-events-none ring-2 ring-green-400 rounded-sm" />
      )}

      <ReactFlow
        nodes={nodes} edges={edges} nodeTypes={nodeTypes} edgeTypes={edgeTypes}
        onNodesChange={onNodesChange} onEdgesChange={onEdgesChange} onConnect={onConnect}
        onNodeClick={onNodeClick} onPaneClick={onPaneClick}
        onNodeDrag={onNodeDrag} onNodeDragStop={onNodeDragStop}
        fitView fitViewOptions={{ padding: 0.2 }}
        defaultEdgeOptions={{ type: 'smoothstep', animated: false, style: { stroke: '#5a8cc8', strokeWidth: 2.5 } }}
      >
        <Background variant={BackgroundVariant.Lines} gap={20} color="#b4dcb4" />
        <Controls />
      </ReactFlow>
    </div>
  );
}

export interface FlowCanvasProps {
  nodes: Node[]; edges: Edge[]; onNodesChange: OnNodesChange; onEdgesChange: OnEdgesChange;
  selectedNodeId: string | null; setSelectedNodeId: (id: string | null) => void;
  onSetNodes: (fn: (nodes: Node[]) => Node[]) => void;
  onSetEdges: (fn: (edges: Edge[]) => Edge[]) => void;
}

export function FlowCanvas(props: FlowCanvasProps) {
  return (
    <ReactFlowProvider>
      <FlowCanvasContent {...props} />
    </ReactFlowProvider>
  );
}
