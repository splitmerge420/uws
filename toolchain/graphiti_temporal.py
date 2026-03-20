"""
Graphiti-style Temporal Knowledge Graph for Aluminum OS.

Implements a lightweight temporal knowledge graph alongside ChromaDB's semantic memory.
Tracks causal relationships, event timelines, and temporal queries without requiring
external Graphiti dependency.
"""

import json
import uuid
from dataclasses import dataclass, asdict, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional, List, Dict, Tuple, Any
import re
from enum import Enum
import argparse


# ============================================================================
# Data Models
# ============================================================================

class NodeType(Enum):
    """Types of nodes in the temporal knowledge graph."""
    EVENT = "event"
    ENTITY = "entity"
    INSIGHT = "insight"
    DECISION = "decision"
    OUTCOME = "outcome"
    STAGE_RUN = "stage_run"
    PIPELINE_RUN = "pipeline_run"
    FIX_APPLICATION = "fix_application"
    VIOLATION = "violation"
    CONSENT = "consent"
    CUSTOM = "custom"


@dataclass
class TemporalNode:
    """A node in the temporal knowledge graph."""
    id: str
    type: str  # NodeType value
    data: Dict[str, Any]
    created_at: str  # ISO format
    updated_at: str  # ISO format
    version: int = 1

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @staticmethod
    def from_dict(data: Dict[str, Any]) -> 'TemporalNode':
        return TemporalNode(**data)


@dataclass
class TemporalEdge:
    """An edge representing a temporal relationship or causality."""
    source_id: str
    target_id: str
    relationship: str  # e.g., "caused", "followed", "depends_on"
    timestamp: str  # ISO format
    metadata: Dict[str, Any] = field(default_factory=dict)
    confidence: float = 1.0  # 0.0 to 1.0

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @staticmethod
    def from_dict(data: Dict[str, Any]) -> 'TemporalEdge':
        return TemporalEdge(**data)


# ============================================================================
# Temporal Knowledge Graph
# ============================================================================

class TemporalKnowledgeGraph:
    """Lightweight temporal knowledge graph for tracking events and causality."""

    def __init__(self):
        self.nodes: Dict[str, TemporalNode] = {}
        self.edges: List[TemporalEdge] = []
        self._node_index: Dict[str, List[str]] = {}  # type -> [node_ids]
        self._created_at = datetime.now().isoformat()

    def add_node(self, node_type: str, data: Dict[str, Any]) -> str:
        """
        Add a node to the graph.

        Args:
            node_type: Type of node (from NodeType enum)
            data: Node data/properties

        Returns:
            node_id: Unique identifier for the node
        """
        node_id = str(uuid.uuid4())
        now = datetime.now().isoformat()

        node = TemporalNode(
            id=node_id,
            type=node_type,
            data=data,
            created_at=now,
            updated_at=now,
            version=1
        )

        self.nodes[node_id] = node

        # Index by type
        if node_type not in self._node_index:
            self._node_index[node_type] = []
        self._node_index[node_type].append(node_id)

        return node_id

    def add_edge(
        self,
        source_id: str,
        target_id: str,
        relationship: str,
        metadata: Optional[Dict[str, Any]] = None,
        confidence: float = 1.0
    ) -> None:
        """
        Add an edge (relationship) between two nodes.

        Args:
            source_id: Source node ID
            target_id: Target node ID
            relationship: Type of relationship (e.g., "caused", "followed")
            metadata: Optional relationship metadata
            confidence: Confidence score (0.0 to 1.0)
        """
        if source_id not in self.nodes or target_id not in self.nodes:
            raise ValueError(f"One or both nodes don't exist: {source_id}, {target_id}")

        edge = TemporalEdge(
            source_id=source_id,
            target_id=target_id,
            relationship=relationship,
            timestamp=datetime.now().isoformat(),
            metadata=metadata or {},
            confidence=max(0.0, min(1.0, confidence))
        )

        self.edges.append(edge)

    def query_temporal(self, start_time: str, end_time: str) -> List[TemporalEdge]:
        """
        Query all edges (events) within a time range.

        Args:
            start_time: ISO format start timestamp
            end_time: ISO format end timestamp

        Returns:
            List of edges in the time range
        """
        try:
            start = datetime.fromisoformat(start_time)
            end = datetime.fromisoformat(end_time)
        except ValueError:
            return []

        results = []
        for edge in self.edges:
            try:
                edge_time = datetime.fromisoformat(edge.timestamp)
                if start <= edge_time <= end:
                    results.append(edge)
            except ValueError:
                continue

        return sorted(results, key=lambda e: e.timestamp)

    def query_causal(self, node_id: str) -> List[TemporalEdge]:
        """
        Query all edges where this node caused something (outgoing causal edges).

        Args:
            node_id: Source node ID

        Returns:
            List of outgoing edges
        """
        if node_id not in self.nodes:
            return []

        results = [e for e in self.edges if e.source_id == node_id]
        return sorted(results, key=lambda e: e.timestamp, reverse=True)

    def query_semantic(self, query_text: str, top_k: int = 5) -> List[TemporalNode]:
        """
        Simple keyword-based semantic search (no embeddings required).

        Args:
            query_text: Search query
            top_k: Maximum results to return

        Returns:
            List of matching nodes
        """
        query_lower = query_text.lower()
        query_words = set(re.findall(r'\w+', query_lower))

        scored_nodes = []

        for node in self.nodes.values():
            # Search in data fields
            score = 0
            data_str = json.dumps(node.data).lower()
            data_words = set(re.findall(r'\w+', data_str))

            # Count matching words
            matches = query_words & data_words
            score = len(matches)

            # Also check node type
            if query_words & {node.type.lower()}:
                score += 2

            if score > 0:
                scored_nodes.append((node, score))

        # Sort by score (descending)
        scored_nodes.sort(key=lambda x: x[1], reverse=True)

        return [node for node, _ in scored_nodes[:top_k]]

    def get_timeline(self, node_id: str) -> List[TemporalEdge]:
        """
        Get the complete history/timeline of a node.

        Args:
            node_id: Node ID

        Returns:
            Chronologically sorted edges involving this node
        """
        if node_id not in self.nodes:
            return []

        # All edges where this node is involved
        involved_edges = [
            e for e in self.edges
            if e.source_id == node_id or e.target_id == node_id
        ]

        return sorted(involved_edges, key=lambda e: e.timestamp)

    def save(self, path: Path) -> None:
        """Save graph to JSON file."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            "created_at": self._created_at,
            "nodes": {nid: n.to_dict() for nid, n in self.nodes.items()},
            "edges": [e.to_dict() for e in self.edges]
        }

        with open(path, 'w') as f:
            json.dump(data, f, indent=2)

    @staticmethod
    def load(path: Path) -> 'TemporalKnowledgeGraph':
        """Load graph from JSON file."""
        path = Path(path)

        if not path.exists():
            raise FileNotFoundError(f"Graph file not found: {path}")

        with open(path, 'r') as f:
            data = json.load(f)

        graph = TemporalKnowledgeGraph()
        graph._created_at = data.get("created_at", datetime.now().isoformat())

        # Load nodes
        for node_data in data.get("nodes", {}).values():
            node = TemporalNode.from_dict(node_data)
            graph.nodes[node.id] = node

            if node.type not in graph._node_index:
                graph._node_index[node.type] = []
            graph._node_index[node.type].append(node.id)

        # Load edges
        for edge_data in data.get("edges", []):
            edge = TemporalEdge.from_dict(edge_data)
            graph.edges.append(edge)

        return graph

    def get_stats(self) -> Dict[str, Any]:
        """Get graph statistics."""
        node_types = {}
        for node in self.nodes.values():
            node_types[node.type] = node_types.get(node.type, 0) + 1

        relationship_types = {}
        for edge in self.edges:
            rel = edge.relationship
            relationship_types[rel] = relationship_types.get(rel, 0) + 1

        # Compute time range
        if self.nodes:
            times = [
                datetime.fromisoformat(n.created_at)
                for n in self.nodes.values()
            ]
            time_range = {
                "earliest": min(times).isoformat(),
                "latest": max(times).isoformat()
            }
        else:
            time_range = None

        return {
            "total_nodes": len(self.nodes),
            "total_edges": len(self.edges),
            "node_types": node_types,
            "relationship_types": relationship_types,
            "time_range": time_range,
            "created_at": self._created_at
        }


# ============================================================================
# Memory Integration Layer
# ============================================================================

class MemoryIntegrationLayer:
    """Bridges ChromaDB semantic memory with Graphiti temporal memory."""

    def __init__(
        self,
        temporal_graph: TemporalKnowledgeGraph,
        chroma_client: Optional[Any] = None,
        collection_name: str = "events"
    ):
        """
        Initialize the integration layer.

        Args:
            temporal_graph: TemporalKnowledgeGraph instance
            chroma_client: Optional ChromaDB client
            collection_name: ChromaDB collection name
        """
        self.graph = temporal_graph
        self.chroma_client = chroma_client
        self.collection = None

        if chroma_client is not None:
            try:
                self.collection = chroma_client.get_or_create_collection(
                    name=collection_name,
                    metadata={"hnsw:space": "cosine"}
                )
            except Exception as e:
                print(f"Warning: Could not initialize ChromaDB collection: {e}")
                self.collection = None

    def store_event(
        self,
        event_type: str,
        data: Dict[str, Any],
        caused_by: Optional[str] = None
    ) -> str:
        """
        Store an event in both temporal graph and ChromaDB.

        Args:
            event_type: Type of event
            data: Event data
            caused_by: Optional node ID that caused this event

        Returns:
            node_id: The created node ID
        """
        # Add to temporal graph
        node_id = self.graph.add_node(event_type, data)

        # Create causal edge if applicable
        if caused_by is not None and caused_by in self.graph.nodes:
            self.graph.add_edge(
                source_id=caused_by,
                target_id=node_id,
                relationship="caused",
                metadata={"automatic": True}
            )

        # Add to ChromaDB if available
        if self.collection is not None:
            try:
                # Create a text representation for ChromaDB
                text = json.dumps(data)

                self.collection.add(
                    ids=[node_id],
                    documents=[text],
                    metadatas=[{
                        "event_type": event_type,
                        "created_at": self.graph.nodes[node_id].created_at,
                        "caused_by": caused_by or "none"
                    }]
                )
            except Exception as e:
                print(f"Warning: Could not store event in ChromaDB: {e}")

        return node_id

    def query_hybrid(
        self,
        query_text: str,
        time_range: Optional[Tuple[str, str]] = None,
        top_k: int = 5
    ) -> Dict[str, Any]:
        """
        Query both ChromaDB and temporal graph, merging results.

        Args:
            query_text: Search query
            time_range: Optional (start_time, end_time) tuple
            top_k: Maximum results

        Returns:
            Dict with combined results
        """
        results = {
            "temporal_nodes": [],
            "temporal_edges": [],
            "chroma_results": []
        }

        # Query temporal graph
        semantic_nodes = self.graph.query_semantic(query_text, top_k=top_k)
        results["temporal_nodes"] = [n.to_dict() for n in semantic_nodes]

        # Apply temporal filter if specified
        if time_range:
            start_time, end_time = time_range
            temporal_edges = self.graph.query_temporal(start_time, end_time)
        else:
            temporal_edges = self.graph.edges

        results["temporal_edges"] = [e.to_dict() for e in temporal_edges[:top_k]]

        # Query ChromaDB if available
        if self.collection is not None:
            try:
                chroma_results = self.collection.query(
                    query_texts=[query_text],
                    n_results=top_k
                )
                if chroma_results and chroma_results.get("documents"):
                    results["chroma_results"] = chroma_results["documents"][0]
            except Exception as e:
                print(f"Warning: ChromaDB query failed: {e}")

        return results


# ============================================================================
# Pipeline Temporal Tracker
# ============================================================================

class PipelineTemporalTracker:
    """Hooks into spheres_pipeline.py to track execution as temporal events."""

    def __init__(self, memory_layer: MemoryIntegrationLayer):
        """
        Initialize the pipeline tracker.

        Args:
            memory_layer: MemoryIntegrationLayer instance
        """
        self.memory = memory_layer
        self._pipeline_run_id: Optional[str] = None
        self._stage_run_ids: Dict[str, str] = {}

    def start_pipeline_run(self, pipeline_name: str, metadata: Optional[Dict] = None) -> str:
        """Record the start of a pipeline run."""
        data = {
            "pipeline_name": pipeline_name,
            "status": "running",
            "metadata": metadata or {}
        }

        self._pipeline_run_id = self.memory.store_event(
            event_type=NodeType.PIPELINE_RUN.value,
            data=data
        )

        return self._pipeline_run_id

    def end_pipeline_run(self, status: str, results: Optional[Dict] = None) -> None:
        """Record the completion of a pipeline run."""
        if self._pipeline_run_id is None:
            return

        node = self.memory.graph.nodes[self._pipeline_run_id]
        node.data["status"] = status
        node.data["results"] = results or {}
        node.updated_at = datetime.now().isoformat()
        node.version += 1

    def record_stage(
        self,
        stage_name: str,
        status: str,
        output: Optional[Dict] = None
    ) -> str:
        """Record a stage execution."""
        data = {
            "stage_name": stage_name,
            "status": status,
            "output": output or {}
        }

        stage_id = self.memory.store_event(
            event_type=NodeType.STAGE_RUN.value,
            data=data,
            caused_by=self._pipeline_run_id
        )

        self._stage_run_ids[stage_name] = stage_id
        return stage_id

    def record_fix_application(
        self,
        fix_name: str,
        applied_to: str,
        success: bool,
        details: Optional[Dict] = None
    ) -> str:
        """Record a fix application as causal chain."""
        data = {
            "fix_name": fix_name,
            "applied_to": applied_to,
            "success": success,
            "details": details or {}
        }

        fix_id = self.memory.store_event(
            event_type=NodeType.FIX_APPLICATION.value,
            data=data,
            caused_by=self._pipeline_run_id
        )

        # If there's a relevant stage, link fix to stage outcome
        for stage_name, stage_id in self._stage_run_ids.items():
            if applied_to.lower() in stage_name.lower():
                self.memory.graph.add_edge(
                    source_id=fix_id,
                    target_id=stage_id,
                    relationship="remedies",
                    metadata={"success": success}
                )

        return fix_id

    def record_violation(
        self,
        violation_type: str,
        severity: str,
        context: Optional[Dict] = None
    ) -> str:
        """Record a consent or policy violation."""
        data = {
            "violation_type": violation_type,
            "severity": severity,
            "context": context or {}
        }

        violation_id = self.memory.store_event(
            event_type=NodeType.VIOLATION.value,
            data=data,
            caused_by=self._pipeline_run_id
        )

        return violation_id


# ============================================================================
# CLI
# ============================================================================

def parse_time_range(time_range_str: str) -> Tuple[str, str]:
    """
    Parse time range string like "7d", "24h", "30m".

    Returns:
        (start_time_iso, end_time_iso)
    """
    now = datetime.now()
    match = re.match(r'(\d+)([dhm])', time_range_str)

    if not match:
        # Default to last 24 hours
        start = now - timedelta(hours=24)
    else:
        amount = int(match.group(1))
        unit = match.group(2)

        if unit == 'd':
            start = now - timedelta(days=amount)
        elif unit == 'h':
            start = now - timedelta(hours=amount)
        else:  # 'm'
            start = now - timedelta(minutes=amount)

    return start.isoformat(), now.isoformat()


def main():
    parser = argparse.ArgumentParser(
        description="Temporal Knowledge Graph CLI for Aluminum OS"
    )
    parser.add_argument(
        "--add-event",
        type=str,
        help="Add an event to the graph"
    )
    parser.add_argument(
        "--type",
        type=str,
        default="custom",
        help="Event type"
    )
    parser.add_argument(
        "--timeline",
        action="store_true",
        help="Show recent events"
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Show graph statistics"
    )
    parser.add_argument(
        "--query",
        type=str,
        help="Semantic query"
    )
    parser.add_argument(
        "--time-range",
        type=str,
        default="24h",
        help="Time range (e.g., 7d, 24h, 30m)"
    )
    parser.add_argument(
        "--graph-file",
        type=str,
        default=".temporal_graph.json",
        help="Path to graph file"
    )

    args = parser.parse_args()
    graph_path = Path(args.graph_file)

    # Load or create graph
    if graph_path.exists():
        graph = TemporalKnowledgeGraph.load(graph_path)
    else:
        graph = TemporalKnowledgeGraph()

    # Handle commands
    if args.add_event:
        node_id = graph.add_node(args.type, {"message": args.add_event})
        print(f"Added event: {node_id}")
        graph.save(graph_path)

    elif args.timeline:
        print("\nRecent Events:")
        print("-" * 60)

        # Get recent edges
        recent_edges = sorted(graph.edges, key=lambda e: e.timestamp, reverse=True)[:10]

        for edge in recent_edges:
            source = graph.nodes.get(edge.source_id)
            target = graph.nodes.get(edge.target_id)
            source_name = source.data.get("message", source.type) if source else "unknown"
            target_name = target.data.get("message", target.type) if target else "unknown"

            timestamp = edge.timestamp
            print(f"{timestamp}: {source_name} -> {target_name} ({edge.relationship})")

        if not recent_edges:
            print("No events found.")

    elif args.stats:
        stats = graph.get_stats()
        print("\nGraph Statistics:")
        print("-" * 60)
        print(f"Total Nodes: {stats['total_nodes']}")
        print(f"Total Edges: {stats['total_edges']}")
        print(f"\nNode Types:")
        for node_type, count in stats["node_types"].items():
            print(f"  {node_type}: {count}")
        print(f"\nRelationship Types:")
        for rel_type, count in stats["relationship_types"].items():
            print(f"  {rel_type}: {count}")

        if stats["time_range"]:
            print(f"\nTime Range:")
            print(f"  Earliest: {stats['time_range']['earliest']}")
            print(f"  Latest: {stats['time_range']['latest']}")

    elif args.query:
        start_time, end_time = parse_time_range(args.time_range)

        # Temporal query
        edges = graph.query_temporal(start_time, end_time)

        # Semantic query
        nodes = graph.query_semantic(args.query, top_k=5)

        print(f"\nQuery Results for '{args.query}' (last {args.time_range}):")
        print("-" * 60)

        if nodes:
            print(f"\nMatching Nodes ({len(nodes)}):")
            for node in nodes:
                print(f"  [{node.type}] {node.id}")
                print(f"    Data: {json.dumps(node.data, indent=6)}")
        else:
            print("No matching nodes found.")

        if edges:
            print(f"\nEvents in Time Range ({len(edges)}):")
            for edge in edges:
                source = graph.nodes.get(edge.source_id)
                target = graph.nodes.get(edge.target_id)
                source_name = source.data.get("message", source.type) if source else "unknown"
                target_name = target.data.get("message", target.type) if target else "unknown"

                print(f"  {edge.timestamp}: {source_name} -> {target_name}")
        else:
            print("No events found in time range.")


if __name__ == "__main__":
    main()
