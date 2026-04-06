import { useCallback } from 'react';
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
import { BlockType, createDefaultConfig } from '../types';
import { dragStore } from '../store/dragStore';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: Record<string, any> = { block: BlockNode };

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

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const blockType = dragStore.get();
      if (!blockType) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const newNode: Node = {
        id: crypto.randomUUID(),
        type: 'block',
        position,
        data: {
          blockType,
          config: createDefaultConfig(blockType),
        },
      };

      onSetNodes((nds) => [...nds, newNode]);
      dragStore.set(null);
    },
    [screenToFlowPosition, onSetNodes]
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

  return (
    <div className="w-full h-full" onDrop={onDrop} onDragOver={onDragOver}>
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
