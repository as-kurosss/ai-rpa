import { useCallback, useEffect, useRef, useState } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  addEdge,
  OnConnect,
  Node,
  Edge,
  BackgroundVariant,
  ReactFlowProvider,
  useReactFlow,
  OnNodesChange,
  OnEdgesChange,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { BlockNode } from './BlockNode';
import { BlockType, createDefaultConfig, BLOCK_LABELS, BLOCK_ICONS } from '../types';
import { dragState } from './BlockPalette';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: Record<string, any> = { block: BlockNode };

const VALID_BLOCKS = ['LaunchApp', 'Click', 'TypeText'];

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

function FlowCanvasContent({
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  selectedNodeId,
  setSelectedNodeId,
  onSetNodes,
  onSetEdges,
}: FlowCanvasContentProps) {
  const { screenToFlowPosition } = useReactFlow();
  const containerRef = useRef<HTMLDivElement>(null);
  const handlersRef = useRef({ onSetNodes, screenToFlowPosition });
  const [isOver, setIsOver] = useState(false);
  const [ghostPos, setGhostPos] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => {
    handlersRef.current = { onSetNodes, screenToFlowPosition };
  }, [onSetNodes, screenToFlowPosition]);

  const onConnect: OnConnect = useCallback(
    (connection) => {
      onSetEdges((eds) =>
        addEdge(
          {
            ...connection,
            animated: true,
            style: { stroke: '#5a8cc8', strokeWidth: 2.5 },
          },
          eds
        )
      );
    },
    [onSetEdges]
  );

  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      setSelectedNodeId(node.id);
    },
    [setSelectedNodeId]
  );

  const onPaneClick = useCallback(() => {
    setSelectedNodeId(null);
  }, [setSelectedNodeId]);

  // Track mouse moves globally while dragging from palette
  useEffect(() => {
    let dragging = false;

    const handleMouseMove = (e: MouseEvent) => {
      if (!dragState.isDragging) {
        if (dragging) {
          dragging = false;
          setGhostPos(null);
          setIsOver(false);
        }
        return;
      }
      dragging = true;

      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const over =
        e.clientX >= rect.left &&
        e.clientX <= rect.right &&
        e.clientY >= rect.top &&
        e.clientY <= rect.bottom;
      setIsOver(over);
      if (over) {
        setGhostPos({ x: e.clientX - rect.left, y: e.clientY - rect.top });
      } else {
        setGhostPos(null);
      }
    };

    const handleMouseUp = (e: MouseEvent) => {
      console.log('[FlowCanvas] mouseup, dragging:', dragging, 'blockType:', dragState.blockType);
      if (!dragging || !dragState.blockType) {
        dragState.set(null);
        setGhostPos(null);
        setIsOver(false);
        return;
      }

      if (!containerRef.current) {
        console.warn('[FlowCanvas] No container ref');
        dragState.set(null);
        setGhostPos(null);
        setIsOver(false);
        return;
      }

      const rect = containerRef.current.getBoundingClientRect();
      const over =
        e.clientX >= rect.left &&
        e.clientX <= rect.right &&
        e.clientY >= rect.top &&
        e.clientY <= rect.bottom;

      console.log('[FlowCanvas] mouseup over canvas:', over, 'clientX:', e.clientX, 'clientY:', e.clientY);

      if (over && dragState.blockType && VALID_BLOCKS.includes(dragState.blockType)) {
        const blockType = dragState.blockType as BlockType;
        const position = handlersRef.current.screenToFlowPosition({
          x: e.clientX,
          y: e.clientY,
        });

        console.log('[FlowCanvas] Dropping block:', blockType, 'at', position);

        const newNode: Node = {
          id: crypto.randomUUID(),
          type: 'block',
          position,
          data: {
            blockType,
            config: createDefaultConfig(blockType),
          },
        };

        handlersRef.current.onSetNodes((nds) => [...nds, newNode]);
      }

      dragState.set(null);
      dragging = false;
      setGhostPos(null);
      setIsOver(false);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  return (
    <div className="w-full h-full relative" ref={containerRef}>
      {/* Ghost element shown while dragging */}
      {ghostPos && dragState.blockType && (
        <div
          className="absolute pointer-events-none z-50 opacity-80 bg-white rounded-lg shadow-lg border-2 border-blue-400 px-3 py-2 text-sm font-medium"
          style={{ left: ghostPos.x - 50, top: ghostPos.y - 15 }}
        >
          {BLOCK_ICONS[dragState.blockType]} {BLOCK_LABELS[dragState.blockType]}
        </div>
      )}

      {/* Drop zone highlight */}
      {isOver && dragState.isDragging && (
        <div className="absolute inset-0 z-40 pointer-events-none ring-2 ring-green-400 rounded-sm" />
      )}

      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onPaneClick={onPaneClick}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        defaultEdgeOptions={{
          type: 'smoothstep',
          animated: true,
          style: { stroke: '#5a8cc8', strokeWidth: 2.5 },
        }}
      >
        <Background variant={BackgroundVariant.Lines} gap={20} color="#b4dcb4" />
        <Controls />
        <MiniMap
          nodeColor={(node) => {
            const bt = (node.data as { blockType?: BlockType } | undefined)?.blockType;
            return bt ? '#828282' : '#aaa';
          }}
          maskColor="rgba(0,0,0,0.08)"
          bgColor="#f5f5f5"
        />
      </ReactFlow>
    </div>
  );
}

export interface FlowCanvasProps {
  nodes: Node[];
  edges: Edge[];
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  selectedNodeId: string | null;
  setSelectedNodeId: (id: string | null) => void;
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
