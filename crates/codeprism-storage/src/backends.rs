//! Storage backend implementations

use crate::{
    AnalysisResult, AnalysisStorage, EdgeReference, GraphMetadata, GraphStorage, SerializableEdge,
    SerializableGraph, SerializableNode,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;

/// In-memory graph storage implementation
pub struct InMemoryGraphStorage {
    graphs: Arc<Mutex<HashMap<String, SerializableGraph>>>,
}

impl InMemoryGraphStorage {
    /// Create a new in-memory graph storage
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryGraphStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphStorage for InMemoryGraphStorage {
    async fn store_graph(&self, graph: &SerializableGraph) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        graphs.insert(graph.repo_id.clone(), graph.clone());
        Ok(())
    }

    async fn load_graph(&self, repo_id: &str) -> Result<Option<SerializableGraph>> {
        let graphs = self.graphs.lock().unwrap();
        Ok(graphs.get(repo_id).cloned())
    }

    async fn update_nodes(&self, repo_id: &str, nodes: &[SerializableNode]) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        if let Some(graph) = graphs.get_mut(repo_id) {
            // Update existing nodes or add new ones
            for new_node in nodes {
                if let Some(existing_node) = graph.nodes.iter_mut().find(|n| n.id == new_node.id) {
                    *existing_node = new_node.clone();
                } else {
                    graph.nodes.push(new_node.clone());
                }
            }
        }
        Ok(())
    }

    async fn update_edges(&self, repo_id: &str, edges: &[SerializableEdge]) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        if let Some(graph) = graphs.get_mut(repo_id) {
            // Update existing edges or add new ones
            for new_edge in edges {
                if let Some(existing_edge) = graph.edges.iter_mut().find(|e| {
                    e.source == new_edge.source
                        && e.target == new_edge.target
                        && e.kind == new_edge.kind
                }) {
                    *existing_edge = new_edge.clone();
                } else {
                    graph.edges.push(new_edge.clone());
                }
            }
        }
        Ok(())
    }

    async fn delete_nodes(&self, repo_id: &str, node_ids: &[String]) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        if let Some(graph) = graphs.get_mut(repo_id) {
            graph.nodes.retain(|n| !node_ids.contains(&n.id));
            // Also remove edges that reference deleted nodes
            graph
                .edges
                .retain(|e| !node_ids.contains(&e.source) && !node_ids.contains(&e.target));
        }
        Ok(())
    }

    async fn delete_edges(&self, repo_id: &str, edge_refs: &[EdgeReference]) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        if let Some(graph) = graphs.get_mut(repo_id) {
            graph.edges.retain(|e| {
                !edge_refs
                    .iter()
                    .any(|er| er.source == e.source && er.target == e.target && er.kind == e.kind)
            });
        }
        Ok(())
    }

    async fn get_graph_metadata(&self, repo_id: &str) -> Result<Option<GraphMetadata>> {
        let graphs = self.graphs.lock().unwrap();
        Ok(graphs.get(repo_id).map(|g| g.metadata.clone()))
    }

    async fn update_graph_metadata(&self, repo_id: &str, metadata: &GraphMetadata) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        if let Some(graph) = graphs.get_mut(repo_id) {
            graph.metadata = metadata.clone();
        }
        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        let graphs = self.graphs.lock().unwrap();
        Ok(graphs.keys().cloned().collect())
    }

    async fn delete_graph(&self, repo_id: &str) -> Result<()> {
        let mut graphs = self.graphs.lock().unwrap();
        graphs.remove(repo_id);
        Ok(())
    }

    async fn graph_exists(&self, repo_id: &str) -> Result<bool> {
        let graphs = self.graphs.lock().unwrap();
        Ok(graphs.contains_key(repo_id))
    }
}

/// File-based graph storage implementation
pub struct FileGraphStorage {
    data_path: PathBuf,
}

impl FileGraphStorage {
    /// Create a new file-based graph storage
    pub async fn new(data_path: &Path) -> Result<Self> {
        let storage = Self {
            data_path: data_path.to_path_buf(),
        };

        // Ensure the data directory exists
        fs::create_dir_all(&storage.data_path)
            .await
            .context("Failed to create data directory")?;

        Ok(storage)
    }

    /// Get the file path for a repository's graph
    fn graph_file_path(&self, repo_id: &str) -> PathBuf {
        self.data_path.join(format!("{}.graph.json", repo_id))
    }

    /// Get the file path for a repository's metadata
    fn metadata_file_path(&self, repo_id: &str) -> PathBuf {
        self.data_path.join(format!("{}.metadata.json", repo_id))
    }
}

#[async_trait]
impl GraphStorage for FileGraphStorage {
    async fn store_graph(&self, graph: &SerializableGraph) -> Result<()> {
        let graph_path = self.graph_file_path(&graph.repo_id);
        let metadata_path = self.metadata_file_path(&graph.repo_id);

        // Serialize and write graph
        let graph_json =
            serde_json::to_string_pretty(graph).context("Failed to serialize graph")?;
        fs::write(&graph_path, graph_json)
            .await
            .with_context(|| format!("Failed to write graph to {:?}", graph_path))?;

        // Serialize and write metadata separately for efficiency
        let metadata_json = serde_json::to_string_pretty(&graph.metadata)
            .context("Failed to serialize metadata")?;
        fs::write(&metadata_path, metadata_json)
            .await
            .with_context(|| format!("Failed to write metadata to {:?}", metadata_path))?;

        Ok(())
    }

    async fn load_graph(&self, repo_id: &str) -> Result<Option<SerializableGraph>> {
        let graph_path = self.graph_file_path(repo_id);

        if !graph_path.exists() {
            return Ok(None);
        }

        let graph_json = fs::read_to_string(&graph_path)
            .await
            .with_context(|| format!("Failed to read graph from {:?}", graph_path))?;

        let graph: SerializableGraph =
            serde_json::from_str(&graph_json).context("Failed to deserialize graph")?;

        Ok(Some(graph))
    }

    async fn update_nodes(&self, repo_id: &str, nodes: &[SerializableNode]) -> Result<()> {
        if let Some(mut graph) = self.load_graph(repo_id).await? {
            // Update existing nodes or add new ones
            for new_node in nodes {
                if let Some(existing_node) = graph.nodes.iter_mut().find(|n| n.id == new_node.id) {
                    *existing_node = new_node.clone();
                } else {
                    graph.nodes.push(new_node.clone());
                }
            }

            // Update timestamp
            graph.metadata.last_updated = SystemTime::now();

            self.store_graph(&graph).await?;
        }
        Ok(())
    }

    async fn update_edges(&self, repo_id: &str, edges: &[SerializableEdge]) -> Result<()> {
        if let Some(mut graph) = self.load_graph(repo_id).await? {
            // Update existing edges or add new ones
            for new_edge in edges {
                if let Some(existing_edge) = graph.edges.iter_mut().find(|e| {
                    e.source == new_edge.source
                        && e.target == new_edge.target
                        && e.kind == new_edge.kind
                }) {
                    *existing_edge = new_edge.clone();
                } else {
                    graph.edges.push(new_edge.clone());
                }
            }

            // Update timestamp
            graph.metadata.last_updated = SystemTime::now();

            self.store_graph(&graph).await?;
        }
        Ok(())
    }

    async fn delete_nodes(&self, repo_id: &str, node_ids: &[String]) -> Result<()> {
        if let Some(mut graph) = self.load_graph(repo_id).await? {
            graph.nodes.retain(|n| !node_ids.contains(&n.id));
            // Also remove edges that reference deleted nodes
            graph
                .edges
                .retain(|e| !node_ids.contains(&e.source) && !node_ids.contains(&e.target));

            // Update timestamp
            graph.metadata.last_updated = SystemTime::now();

            self.store_graph(&graph).await?;
        }
        Ok(())
    }

    async fn delete_edges(&self, repo_id: &str, edge_refs: &[EdgeReference]) -> Result<()> {
        if let Some(mut graph) = self.load_graph(repo_id).await? {
            graph.edges.retain(|e| {
                !edge_refs
                    .iter()
                    .any(|er| er.source == e.source && er.target == e.target && er.kind == e.kind)
            });

            // Update timestamp
            graph.metadata.last_updated = SystemTime::now();

            self.store_graph(&graph).await?;
        }
        Ok(())
    }

    async fn get_graph_metadata(&self, repo_id: &str) -> Result<Option<GraphMetadata>> {
        let metadata_path = self.metadata_file_path(repo_id);

        if !metadata_path.exists() {
            return Ok(None);
        }

        let metadata_json = fs::read_to_string(&metadata_path)
            .await
            .with_context(|| format!("Failed to read metadata from {:?}", metadata_path))?;

        let metadata: GraphMetadata =
            serde_json::from_str(&metadata_json).context("Failed to deserialize metadata")?;

        Ok(Some(metadata))
    }

    async fn update_graph_metadata(&self, repo_id: &str, metadata: &GraphMetadata) -> Result<()> {
        let metadata_path = self.metadata_file_path(repo_id);

        let metadata_json =
            serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;
        fs::write(&metadata_path, metadata_json)
            .await
            .with_context(|| format!("Failed to write metadata to {:?}", metadata_path))?;

        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        let mut repos = Vec::new();
        let mut entries = fs::read_dir(&self.data_path)
            .await
            .context("Failed to read data directory")?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".graph.json") {
                    let repo_id = file_name.strip_suffix(".graph.json").unwrap().to_string();
                    repos.push(repo_id);
                }
            }
        }

        Ok(repos)
    }

    async fn delete_graph(&self, repo_id: &str) -> Result<()> {
        let graph_path = self.graph_file_path(repo_id);
        let metadata_path = self.metadata_file_path(repo_id);

        // Remove both files if they exist
        if graph_path.exists() {
            fs::remove_file(&graph_path)
                .await
                .with_context(|| format!("Failed to remove graph file {:?}", graph_path))?;
        }

        if metadata_path.exists() {
            fs::remove_file(&metadata_path)
                .await
                .with_context(|| format!("Failed to remove metadata file {:?}", metadata_path))?;
        }

        Ok(())
    }

    async fn graph_exists(&self, repo_id: &str) -> Result<bool> {
        let graph_path = self.graph_file_path(repo_id);
        Ok(graph_path.exists())
    }
}

/// SQLite-based graph storage implementation
pub struct SqliteGraphStorage {
    #[allow(dead_code)] // TODO: Will be used for database path management
    db_path: PathBuf,
    connection: Arc<AsyncMutex<Connection>>,
}

impl SqliteGraphStorage {
    /// Create a new SQLite-based graph storage
    pub async fn new(data_path: &Path) -> Result<Self> {
        // Ensure the data directory exists
        fs::create_dir_all(data_path)
            .await
            .context("Failed to create data directory")?;

        let db_path = data_path.join("codeprism.db");
        let connection = Connection::open(&db_path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", db_path))?;

        let storage = Self {
            db_path: db_path.clone(),
            connection: Arc::new(AsyncMutex::new(connection)),
        };

        // Initialize database schema
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> Result<()> {
        let conn = self.connection.lock().await;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graphs (
                repo_id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS nodes (
                repo_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                data BLOB NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (repo_id, node_id),
                FOREIGN KEY (repo_id) REFERENCES graphs(repo_id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS edges (
                repo_id TEXT NOT NULL,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                kind TEXT NOT NULL,
                data BLOB NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (repo_id, source, target, kind),
                FOREIGN KEY (repo_id) REFERENCES graphs(repo_id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                repo_id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (repo_id) REFERENCES graphs(repo_id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indices for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_repo_id ON nodes(repo_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_edges_repo_id ON edges(repo_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target)",
            [],
        )?;

        Ok(())
    }

    /// Convert SystemTime to Unix timestamp
    fn system_time_to_timestamp(time: SystemTime) -> i64 {
        time.duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Convert Unix timestamp to SystemTime
    #[allow(dead_code)] // TODO: Will be used for timestamp conversion utilities
    fn timestamp_to_system_time(timestamp: i64) -> SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64)
    }
}

#[async_trait]
impl GraphStorage for SqliteGraphStorage {
    async fn store_graph(&self, graph: &SerializableGraph) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        // Serialize the entire graph
        let graph_data = bincode::serialize(graph).context("Failed to serialize graph")?;

        // Store the graph
        conn.execute(
            "INSERT OR REPLACE INTO graphs (repo_id, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![graph.repo_id, graph_data, now, now],
        )?;

        // Store metadata separately for efficient queries
        let metadata_data =
            bincode::serialize(&graph.metadata).context("Failed to serialize metadata")?;

        conn.execute(
            "INSERT OR REPLACE INTO metadata (repo_id, data, updated_at) VALUES (?1, ?2, ?3)",
            params![graph.repo_id, metadata_data, now],
        )?;

        // Store nodes individually for incremental updates
        for node in &graph.nodes {
            let node_data = bincode::serialize(node).context("Failed to serialize node")?;

            conn.execute(
                "INSERT OR REPLACE INTO nodes (repo_id, node_id, data, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![graph.repo_id, node.id, node_data, now],
            )?;
        }

        // Store edges individually for incremental updates
        for edge in &graph.edges {
            let edge_data = bincode::serialize(edge).context("Failed to serialize edge")?;

            conn.execute(
                "INSERT OR REPLACE INTO edges (repo_id, source, target, kind, data, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![graph.repo_id, edge.source, edge.target, edge.kind, edge_data, now],
            )?;
        }

        Ok(())
    }

    async fn load_graph(&self, repo_id: &str) -> Result<Option<SerializableGraph>> {
        let conn = self.connection.lock().await;

        // Check if graph exists
        let mut stmt = conn.prepare("SELECT 1 FROM graphs WHERE repo_id = ?1")?;
        let exists = stmt.exists([repo_id])?;

        if !exists {
            return Ok(None);
        }

        // Load metadata
        let mut metadata_stmt = conn.prepare("SELECT data FROM metadata WHERE repo_id = ?1")?;
        let metadata_result: Result<Vec<u8>, rusqlite::Error> =
            metadata_stmt.query_row([repo_id], |row| row.get(0));

        let metadata = match metadata_result {
            Ok(metadata_data) => {
                bincode::deserialize(&metadata_data).context("Failed to deserialize metadata")?
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Load nodes
        let mut nodes = Vec::new();
        let mut node_stmt = conn.prepare("SELECT data FROM nodes WHERE repo_id = ?1")?;
        let node_rows = node_stmt.query_map([repo_id], |row| row.get::<_, Vec<u8>>(0))?;

        for row in node_rows {
            let node_data = row?;
            let node: SerializableNode =
                bincode::deserialize(&node_data).context("Failed to deserialize node")?;
            nodes.push(node);
        }

        // Load edges
        let mut edges = Vec::new();
        let mut edge_stmt = conn.prepare("SELECT data FROM edges WHERE repo_id = ?1")?;
        let edge_rows = edge_stmt.query_map([repo_id], |row| row.get::<_, Vec<u8>>(0))?;

        for row in edge_rows {
            let edge_data = row?;
            let edge: SerializableEdge =
                bincode::deserialize(&edge_data).context("Failed to deserialize edge")?;
            edges.push(edge);
        }

        // Construct graph
        let graph = SerializableGraph {
            repo_id: repo_id.to_string(),
            nodes,
            edges,
            metadata,
        };

        Ok(Some(graph))
    }

    async fn update_nodes(&self, repo_id: &str, nodes: &[SerializableNode]) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        for node in nodes {
            let node_data = bincode::serialize(node).context("Failed to serialize node")?;

            conn.execute(
                "INSERT OR REPLACE INTO nodes (repo_id, node_id, data, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![repo_id, node.id, node_data, now],
            )?;
        }

        // Update the graph's updated_at timestamp
        conn.execute(
            "UPDATE graphs SET updated_at = ?1 WHERE repo_id = ?2",
            params![now, repo_id],
        )?;

        Ok(())
    }

    async fn update_edges(&self, repo_id: &str, edges: &[SerializableEdge]) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        for edge in edges {
            let edge_data = bincode::serialize(edge).context("Failed to serialize edge")?;

            conn.execute(
                "INSERT OR REPLACE INTO edges (repo_id, source, target, kind, data, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![repo_id, edge.source, edge.target, edge.kind, edge_data, now],
            )?;
        }

        // Update the graph's updated_at timestamp
        conn.execute(
            "UPDATE graphs SET updated_at = ?1 WHERE repo_id = ?2",
            params![now, repo_id],
        )?;

        Ok(())
    }

    async fn delete_nodes(&self, repo_id: &str, node_ids: &[String]) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        for node_id in node_ids {
            // Delete the node
            conn.execute(
                "DELETE FROM nodes WHERE repo_id = ?1 AND node_id = ?2",
                params![repo_id, node_id],
            )?;

            // Delete edges that reference this node
            conn.execute(
                "DELETE FROM edges WHERE repo_id = ?1 AND (source = ?2 OR target = ?2)",
                params![repo_id, node_id],
            )?;
        }

        // Update the graph's updated_at timestamp
        conn.execute(
            "UPDATE graphs SET updated_at = ?1 WHERE repo_id = ?2",
            params![now, repo_id],
        )?;

        Ok(())
    }

    async fn delete_edges(&self, repo_id: &str, edge_refs: &[EdgeReference]) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        for edge_ref in edge_refs {
            conn.execute(
                "DELETE FROM edges WHERE repo_id = ?1 AND source = ?2 AND target = ?3 AND kind = ?4",
                params![repo_id, edge_ref.source, edge_ref.target, edge_ref.kind],
            )?;
        }

        // Update the graph's updated_at timestamp
        conn.execute(
            "UPDATE graphs SET updated_at = ?1 WHERE repo_id = ?2",
            params![now, repo_id],
        )?;

        Ok(())
    }

    async fn get_graph_metadata(&self, repo_id: &str) -> Result<Option<GraphMetadata>> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT data FROM metadata WHERE repo_id = ?1")?;
        let metadata_result: Result<Vec<u8>, rusqlite::Error> =
            stmt.query_row([repo_id], |row| row.get::<_, Vec<u8>>(0));

        match metadata_result {
            Ok(metadata_data) => {
                let metadata: GraphMetadata = bincode::deserialize(&metadata_data)
                    .context("Failed to deserialize metadata")?;
                Ok(Some(metadata))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_graph_metadata(&self, repo_id: &str, metadata: &GraphMetadata) -> Result<()> {
        let conn = self.connection.lock().await;
        let now = Self::system_time_to_timestamp(SystemTime::now());

        let metadata_data = bincode::serialize(metadata).context("Failed to serialize metadata")?;

        conn.execute(
            "INSERT OR REPLACE INTO metadata (repo_id, data, updated_at) VALUES (?1, ?2, ?3)",
            params![repo_id, metadata_data, now],
        )?;

        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT repo_id FROM graphs ORDER BY repo_id")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut repos = Vec::new();
        for row in rows {
            repos.push(row?);
        }

        Ok(repos)
    }

    async fn delete_graph(&self, repo_id: &str) -> Result<()> {
        let conn = self.connection.lock().await;

        // Delete from graphs table (CASCADE will handle related tables)
        conn.execute("DELETE FROM graphs WHERE repo_id = ?1", [repo_id])?;

        Ok(())
    }

    async fn graph_exists(&self, repo_id: &str) -> Result<bool> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT 1 FROM graphs WHERE repo_id = ?1")?;
        let exists = stmt.exists([repo_id])?;

        Ok(exists)
    }
}

/// Placeholder implementation for Neo4j backend
pub struct Neo4jGraphStorage;

impl Neo4jGraphStorage {
    pub async fn new(_connection_string: &Option<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl GraphStorage for Neo4jGraphStorage {
    async fn store_graph(&self, _graph: &SerializableGraph) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn load_graph(&self, _repo_id: &str) -> Result<Option<SerializableGraph>> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn update_nodes(&self, _repo_id: &str, _nodes: &[SerializableNode]) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn update_edges(&self, _repo_id: &str, _edges: &[SerializableEdge]) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn delete_nodes(&self, _repo_id: &str, _node_ids: &[String]) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn delete_edges(&self, _repo_id: &str, _edge_refs: &[EdgeReference]) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn get_graph_metadata(&self, _repo_id: &str) -> Result<Option<GraphMetadata>> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn update_graph_metadata(&self, _repo_id: &str, _metadata: &GraphMetadata) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn delete_graph(&self, _repo_id: &str) -> Result<()> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }

    async fn graph_exists(&self, _repo_id: &str) -> Result<bool> {
        anyhow::bail!("Neo4j backend not implemented yet")
    }
}

/// In-memory analysis storage
pub struct InMemoryAnalysisStorage {
    results: Arc<Mutex<HashMap<String, AnalysisResult>>>,
}

impl InMemoryAnalysisStorage {
    /// Create a new in-memory analysis storage
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryAnalysisStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AnalysisStorage for InMemoryAnalysisStorage {
    async fn store_analysis(&self, result: &AnalysisResult) -> Result<()> {
        let mut results = self.results.lock().unwrap();
        results.insert(result.id.clone(), result.clone());
        Ok(())
    }

    async fn load_analysis(&self, result_id: &str) -> Result<Option<AnalysisResult>> {
        let results = self.results.lock().unwrap();
        Ok(results.get(result_id).cloned())
    }

    async fn find_analysis(
        &self,
        repo_id: &str,
        analysis_type: Option<&str>,
        since: Option<SystemTime>,
    ) -> Result<Vec<AnalysisResult>> {
        let results = self.results.lock().unwrap();
        let filtered: Vec<AnalysisResult> = results
            .values()
            .filter(|r| {
                r.repo_id == repo_id
                    && analysis_type.is_none_or(|t| r.analysis_type == t)
                    && since.is_none_or(|s| r.timestamp >= s)
            })
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn delete_analysis(&self, result_id: &str) -> Result<()> {
        let mut results = self.results.lock().unwrap();
        results.remove(result_id);
        Ok(())
    }

    async fn cleanup_old_results(&self, older_than: SystemTime) -> Result<usize> {
        let mut results = self.results.lock().unwrap();
        let keys_to_remove: Vec<String> = results
            .iter()
            .filter(|(_, r)| r.timestamp < older_than)
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            results.remove(&key);
        }

        Ok(count)
    }
}

/// File analysis storage placeholder
pub struct FileAnalysisStorage;

impl FileAnalysisStorage {
    /// Create a new file-based analysis storage
    pub async fn new(_data_path: &Path) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl AnalysisStorage for FileAnalysisStorage {
    async fn store_analysis(&self, _result: &AnalysisResult) -> Result<()> {
        anyhow::bail!("Not implemented")
    }

    async fn load_analysis(&self, _result_id: &str) -> Result<Option<AnalysisResult>> {
        anyhow::bail!("Not implemented")
    }

    async fn find_analysis(
        &self,
        _repo_id: &str,
        _analysis_type: Option<&str>,
        _since: Option<SystemTime>,
    ) -> Result<Vec<AnalysisResult>> {
        anyhow::bail!("Not implemented")
    }

    async fn delete_analysis(&self, _result_id: &str) -> Result<()> {
        anyhow::bail!("Not implemented")
    }

    async fn cleanup_old_results(&self, _older_than: SystemTime) -> Result<usize> {
        anyhow::bail!("Not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::SerializableSpan;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_graph() -> SerializableGraph {
        let mut graph = SerializableGraph::new("test_repo".to_string());

        // Add test nodes
        let node1 = SerializableNode::new(
            "node1".to_string(),
            "TestFunction".to_string(),
            "function".to_string(),
            PathBuf::from("test.rs"),
            SerializableSpan {
                start_byte: 0,
                end_byte: 100,
                start_line: 1,
                end_line: 5,
                start_column: 0,
                end_column: 10,
            },
        );

        let node2 = SerializableNode::new(
            "node2".to_string(),
            "TestStruct".to_string(),
            "struct".to_string(),
            PathBuf::from("test.rs"),
            SerializableSpan {
                start_byte: 101,
                end_byte: 200,
                start_line: 6,
                end_line: 10,
                start_column: 0,
                end_column: 15,
            },
        );

        graph.add_node(node1);
        graph.add_node(node2);

        // Add test edges
        let edge = SerializableEdge::new(
            "node1".to_string(),
            "node2".to_string(),
            "calls".to_string(),
        );
        graph.add_edge(edge);

        graph.update_metadata();
        graph
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryGraphStorage::new();
        let graph = create_test_graph();

        // Test store and load
        storage.store_graph(&graph).await.unwrap();
        let loaded = storage.load_graph("test_repo").await.unwrap();
        assert!(loaded.is_some());
        let loaded_graph = loaded.unwrap();
        assert_eq!(loaded_graph.repo_id, "test_repo");
        assert_eq!(loaded_graph.nodes.len(), 2);
        assert_eq!(loaded_graph.edges.len(), 1);

        // Test update nodes
        let new_node = SerializableNode::new(
            "node3".to_string(),
            "NewFunction".to_string(),
            "function".to_string(),
            PathBuf::from("new.rs"),
            SerializableSpan {
                start_byte: 0,
                end_byte: 50,
                start_line: 1,
                end_line: 3,
                start_column: 0,
                end_column: 5,
            },
        );
        storage
            .update_nodes("test_repo", &[new_node])
            .await
            .unwrap();

        let updated = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(updated.nodes.len(), 3);

        // Test delete nodes
        storage
            .delete_nodes("test_repo", &["node1".to_string()])
            .await
            .unwrap();
        let after_delete = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(after_delete.nodes.len(), 2);
        assert_eq!(after_delete.edges.len(), 0); // Edge should be deleted too

        // Test list repositories
        let repos = storage.list_repositories().await.unwrap();
        assert!(repos.contains(&"test_repo".to_string()));

        // Test graph exists
        assert!(storage.graph_exists("test_repo").await.unwrap());
        assert!(!storage.graph_exists("nonexistent").await.unwrap());

        // Test delete graph
        storage.delete_graph("test_repo").await.unwrap();
        assert!(!storage.graph_exists("test_repo").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = FileGraphStorage::new(temp_dir.path()).await.unwrap();
        let graph = create_test_graph();

        // Test store and load
        storage.store_graph(&graph).await.unwrap();
        let loaded = storage.load_graph("test_repo").await.unwrap();
        assert!(loaded.is_some());
        let loaded_graph = loaded.unwrap();
        assert_eq!(loaded_graph.repo_id, "test_repo");
        assert_eq!(loaded_graph.nodes.len(), 2);
        assert_eq!(loaded_graph.edges.len(), 1);

        // Test metadata operations
        let metadata = storage.get_graph_metadata("test_repo").await.unwrap();
        assert!(metadata.is_some());

        let mut new_metadata = metadata.unwrap();
        new_metadata.version = 42;
        storage
            .update_graph_metadata("test_repo", &new_metadata)
            .await
            .unwrap();

        let updated_metadata = storage
            .get_graph_metadata("test_repo")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_metadata.version, 42);

        // Test incremental updates
        let new_node = SerializableNode::new(
            "node3".to_string(),
            "NewFunction".to_string(),
            "function".to_string(),
            PathBuf::from("new.rs"),
            SerializableSpan {
                start_byte: 0,
                end_byte: 50,
                start_line: 1,
                end_line: 3,
                start_column: 0,
                end_column: 5,
            },
        );
        storage
            .update_nodes("test_repo", &[new_node])
            .await
            .unwrap();

        let updated = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(updated.nodes.len(), 3);

        // Test edge updates
        let new_edge = SerializableEdge::new(
            "node2".to_string(),
            "node3".to_string(),
            "references".to_string(),
        );
        storage
            .update_edges("test_repo", &[new_edge])
            .await
            .unwrap();

        let updated = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(updated.edges.len(), 2);

        // Test list repositories
        let repos = storage.list_repositories().await.unwrap();
        assert!(repos.contains(&"test_repo".to_string()));

        // Test graph exists
        assert!(storage.graph_exists("test_repo").await.unwrap());
        assert!(!storage.graph_exists("nonexistent").await.unwrap());

        // Test delete operations
        let edge_refs = vec![EdgeReference {
            source: "node1".to_string(),
            target: "node2".to_string(),
            kind: "calls".to_string(),
        }];
        storage.delete_edges("test_repo", &edge_refs).await.unwrap();

        let after_edge_delete = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(after_edge_delete.edges.len(), 1);

        // Test delete graph
        storage.delete_graph("test_repo").await.unwrap();
        assert!(!storage.graph_exists("test_repo").await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = SqliteGraphStorage::new(temp_dir.path()).await.unwrap();
        let graph = create_test_graph();

        // Test store and load
        storage.store_graph(&graph).await.unwrap();
        let loaded = storage.load_graph("test_repo").await.unwrap();
        assert!(loaded.is_some());
        let loaded_graph = loaded.unwrap();
        assert_eq!(loaded_graph.repo_id, "test_repo");
        assert_eq!(loaded_graph.nodes.len(), 2);
        assert_eq!(loaded_graph.edges.len(), 1);

        // Test metadata operations
        let metadata = storage.get_graph_metadata("test_repo").await.unwrap();
        assert!(metadata.is_some());

        let mut new_metadata = metadata.unwrap();
        new_metadata.version = 42;
        storage
            .update_graph_metadata("test_repo", &new_metadata)
            .await
            .unwrap();

        let updated_metadata = storage
            .get_graph_metadata("test_repo")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_metadata.version, 42);

        // Test incremental updates
        let new_node = SerializableNode::new(
            "node3".to_string(),
            "NewFunction".to_string(),
            "function".to_string(),
            PathBuf::from("new.rs"),
            SerializableSpan {
                start_byte: 0,
                end_byte: 50,
                start_line: 1,
                end_line: 3,
                start_column: 0,
                end_column: 5,
            },
        );
        storage
            .update_nodes("test_repo", &[new_node])
            .await
            .unwrap();

        let updated = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(updated.nodes.len(), 3);

        // Test edge updates
        let new_edge = SerializableEdge::new(
            "node2".to_string(),
            "node3".to_string(),
            "references".to_string(),
        );
        storage
            .update_edges("test_repo", &[new_edge])
            .await
            .unwrap();

        let updated = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(updated.edges.len(), 2);

        // Test list repositories
        let repos = storage.list_repositories().await.unwrap();
        assert!(repos.contains(&"test_repo".to_string()));

        // Test graph exists
        assert!(storage.graph_exists("test_repo").await.unwrap());
        assert!(!storage.graph_exists("nonexistent").await.unwrap());

        // Test delete operations
        let edge_refs = vec![EdgeReference {
            source: "node1".to_string(),
            target: "node2".to_string(),
            kind: "calls".to_string(),
        }];
        storage.delete_edges("test_repo", &edge_refs).await.unwrap();

        let after_edge_delete = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(after_edge_delete.edges.len(), 1);

        // Test delete nodes (should also remove related edges)
        storage
            .delete_nodes("test_repo", &["node3".to_string()])
            .await
            .unwrap();
        let after_node_delete = storage.load_graph("test_repo").await.unwrap().unwrap();
        assert_eq!(after_node_delete.nodes.len(), 2);
        assert_eq!(after_node_delete.edges.len(), 0); // Edge to node3 should be deleted

        // Test delete graph
        storage.delete_graph("test_repo").await.unwrap();
        assert!(!storage.graph_exists("test_repo").await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_error_handling() {
        // Test FileGraphStorage with invalid path
        let invalid_path = PathBuf::from("/invalid/path/that/should/not/exist");
        let result = FileGraphStorage::new(&invalid_path).await;
        assert!(result.is_err());

        // Test loading non-existent graph
        let temp_dir = tempdir().unwrap();
        let storage = FileGraphStorage::new(temp_dir.path()).await.unwrap();
        let result = storage.load_graph("nonexistent").await.unwrap();
        assert!(result.is_none());

        // Test SQLite with read-only directory (if possible)
        let temp_dir = tempdir().unwrap();
        let storage = SqliteGraphStorage::new(temp_dir.path()).await.unwrap();
        let result = storage.load_graph("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use tokio::task;

        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(SqliteGraphStorage::new(temp_dir.path()).await.unwrap());

        // Test concurrent writes
        let mut handles = Vec::new();
        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = task::spawn(async move {
                let mut graph = create_test_graph();
                graph.repo_id = format!("repo_{}", i);
                storage_clone.store_graph(&graph).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all writes to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all graphs were stored
        let repos = storage.list_repositories().await.unwrap();
        assert_eq!(repos.len(), 10);

        // Test concurrent reads
        let mut handles = Vec::new();
        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = task::spawn(async move {
                let repo_id = format!("repo_{}", i);
                let graph = storage_clone.load_graph(&repo_id).await.unwrap();
                assert!(graph.is_some());
                assert_eq!(graph.unwrap().repo_id, repo_id);
            });
            handles.push(handle);
        }

        // Wait for all reads to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
