//! Session management for MCP tools
//!
//! Provides session state tracking, analysis history, and workflow stage detection
//! to enable intelligent tool guidance and reduce redundant analysis.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Unique identifier for a session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    /// Generate a new unique session ID
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis();
        let random: u32 = rand::random();
        Self(format!("session_{}_{}", timestamp, random))
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Current workflow stage of analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStage {
    /// Initial exploration and discovery
    Discovery,
    /// Understanding relationships and structure
    Mapping,
    /// Detailed analysis of specific areas
    DeepDive,
    /// Putting findings together
    Synthesis,
}

impl WorkflowStage {
    /// Get appropriate tools for this stage
    pub fn recommended_tools(&self) -> Vec<&'static str> {
        match self {
            WorkflowStage::Discovery => vec![
                "repository_stats",
                "search_content",
                "find_files",
                "content_stats",
            ],
            WorkflowStage::Mapping => vec![
                "search_symbols",
                "find_dependencies",
                "detect_patterns",
                "trace_path",
            ],
            WorkflowStage::DeepDive => vec![
                "explain_symbol",
                "trace_inheritance",
                "analyze_decorators",
                "find_references",
            ],
            WorkflowStage::Synthesis => vec!["analyze_complexity"],
        }
    }

    /// Get next logical stage
    pub fn next_stage(&self) -> Option<WorkflowStage> {
        match self {
            WorkflowStage::Discovery => Some(WorkflowStage::Mapping),
            WorkflowStage::Mapping => Some(WorkflowStage::DeepDive),
            WorkflowStage::DeepDive => Some(WorkflowStage::Synthesis),
            WorkflowStage::Synthesis => None,
        }
    }
}

/// Analysis operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    /// Tool that was called
    pub tool_name: String,
    /// Parameters used
    pub parameters: serde_json::Value,
    /// Timestamp of the analysis
    pub timestamp: u64,
    /// Success or failure
    pub success: bool,
    /// Summary of results
    pub result_summary: Option<String>,
    /// Symbols analyzed (if applicable)
    pub symbols_analyzed: Vec<String>,
}

impl AnalysisRecord {
    /// Create a new analysis record
    pub fn new(
        tool_name: String,
        parameters: serde_json::Value,
        success: bool,
        result_summary: Option<String>,
        symbols_analyzed: Vec<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            tool_name,
            parameters,
            timestamp,
            success,
            result_summary,
            symbols_analyzed,
        }
    }
}

/// History of analysis operations in a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisHistory {
    /// All analysis records in chronological order
    pub records: Vec<AnalysisRecord>,
    /// Symbols that have been analyzed
    pub analyzed_symbols: std::collections::HashSet<String>,
    /// Patterns discovered
    pub discovered_patterns: Vec<String>,
    /// Current focus areas
    pub focus_areas: Vec<String>,
}

impl AnalysisHistory {
    /// Add a new analysis record
    pub fn add_record(&mut self, record: AnalysisRecord) {
        // Update analyzed symbols
        for symbol in &record.symbols_analyzed {
            self.analyzed_symbols.insert(symbol.clone());
        }

        self.records.push(record);
    }

    /// Check if a symbol has been analyzed recently
    pub fn was_recently_analyzed(&self, symbol: &str, within_minutes: u64) -> bool {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            - (within_minutes * 60);

        self.records.iter().any(|record| {
            record.timestamp > cutoff_time && record.symbols_analyzed.contains(&symbol.to_string())
        })
    }

    /// Get recent tools used
    pub fn recent_tools(&self, within_minutes: u64) -> Vec<String> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            - (within_minutes * 60);

        self.records
            .iter()
            .filter(|record| record.timestamp > cutoff_time)
            .map(|record| record.tool_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Detect current workflow stage based on recent activity
    pub fn detect_workflow_stage(&self) -> WorkflowStage {
        let recent_tools = self.recent_tools(30); // Last 30 minutes

        // Count tools by category
        let discovery_tools = [
            "repository_stats",
            "search_content",
            "find_files",
            "content_stats",
        ];
        let mapping_tools = [
            "search_symbols",
            "find_dependencies",
            "detect_patterns",
            "trace_path",
        ];
        let deepdive_tools = [
            "explain_symbol",
            "trace_inheritance",
            "analyze_decorators",
            "find_references",
        ];
        let synthesis_tools = ["analyze_complexity"];

        let discovery_count = recent_tools
            .iter()
            .filter(|t| discovery_tools.contains(&t.as_str()))
            .count();
        let mapping_count = recent_tools
            .iter()
            .filter(|t| mapping_tools.contains(&t.as_str()))
            .count();
        let deepdive_count = recent_tools
            .iter()
            .filter(|t| deepdive_tools.contains(&t.as_str()))
            .count();
        let synthesis_count = recent_tools
            .iter()
            .filter(|t| synthesis_tools.contains(&t.as_str()))
            .count();

        // Determine stage based on dominant activity
        if synthesis_count > 0 {
            WorkflowStage::Synthesis
        } else if deepdive_count > mapping_count && deepdive_count > discovery_count {
            WorkflowStage::DeepDive
        } else if mapping_count > discovery_count {
            WorkflowStage::Mapping
        } else {
            WorkflowStage::Discovery
        }
    }
}

/// Current state of a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique session identifier
    pub id: SessionId,
    /// When the session was created
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Analysis history
    pub history: AnalysisHistory,
    /// Current workflow stage
    pub current_stage: WorkflowStage,
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SessionState {
    /// Create a new session state
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            id: SessionId::new(),
            created_at: now,
            last_activity: now,
            history: AnalysisHistory::default(),
            current_stage: WorkflowStage::Discovery,
            metadata: HashMap::new(),
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Update workflow stage based on recent activity
        self.current_stage = self.history.detect_workflow_stage();
    }

    /// Add an analysis record and update state
    pub fn record_analysis(
        &mut self,
        tool_name: String,
        parameters: serde_json::Value,
        success: bool,
        result_summary: Option<String>,
        symbols_analyzed: Vec<String>,
    ) {
        let record = AnalysisRecord::new(
            tool_name,
            parameters,
            success,
            result_summary,
            symbols_analyzed,
        );

        self.history.add_record(record);
        self.touch();
    }

    /// Check if session is expired (inactive for over 1 hour)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        now - self.last_activity > 3600 // 1 hour
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages multiple sessions and provides session lifecycle management
#[derive(Debug)]
pub struct SessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<SessionId, SessionState>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub fn create_session(&self) -> Result<SessionId> {
        let session = SessionState::new();
        let session_id = session.id.clone();

        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on sessions"))?;

        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Get a session by ID (create if not exists)
    pub fn get_or_create_session(&self, session_id: Option<SessionId>) -> Result<SessionId> {
        match session_id {
            Some(id) => {
                let sessions = self
                    .sessions
                    .read()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on sessions"))?;

                if sessions.contains_key(&id) {
                    Ok(id)
                } else {
                    drop(sessions);
                    self.create_session()
                }
            }
            None => self.create_session(),
        }
    }

    /// Get session state (readonly)
    pub fn get_session(&self, session_id: &SessionId) -> Result<Option<SessionState>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on sessions"))?;

        Ok(sessions.get(session_id).cloned())
    }

    /// Update session with analysis record
    pub fn record_analysis(
        &self,
        session_id: &SessionId,
        tool_name: String,
        parameters: serde_json::Value,
        success: bool,
        result_summary: Option<String>,
        symbols_analyzed: Vec<String>,
    ) -> Result<()> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on sessions"))?;

        if let Some(session) = sessions.get_mut(session_id) {
            session.record_analysis(
                tool_name,
                parameters,
                success,
                result_summary,
                symbols_analyzed,
            );
        }

        Ok(())
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on sessions"))?;

        let initial_count = sessions.len();
        sessions.retain(|_, session| !session.is_expired());

        Ok(initial_count - sessions.len())
    }

    /// Get active session count
    pub fn active_session_count(&self) -> Result<usize> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on sessions"))?;

        Ok(sessions.len())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
        assert!(id1.0.starts_with("session_"));
    }

    #[test]
    fn test_workflow_stage_progression() {
        assert_eq!(
            WorkflowStage::Discovery.next_stage(),
            Some(WorkflowStage::Mapping)
        );
        assert_eq!(
            WorkflowStage::Mapping.next_stage(),
            Some(WorkflowStage::DeepDive)
        );
        assert_eq!(
            WorkflowStage::DeepDive.next_stage(),
            Some(WorkflowStage::Synthesis)
        );
        assert_eq!(WorkflowStage::Synthesis.next_stage(), None);
    }

    #[test]
    fn test_workflow_stage_tools() {
        let discovery_tools = WorkflowStage::Discovery.recommended_tools();
        assert!(discovery_tools.contains(&"repository_stats"));
        assert!(discovery_tools.contains(&"search_content"));
    }

    #[test]
    fn test_analysis_history() {
        let mut history = AnalysisHistory::default();

        let record = AnalysisRecord::new(
            "explain_symbol".to_string(),
            serde_json::json!({"symbol_id": "test123"}),
            true,
            Some("Symbol explained successfully".to_string()),
            vec!["test123".to_string()],
        );

        history.add_record(record);
        assert_eq!(history.records.len(), 1);
        assert!(history.analyzed_symbols.contains("test123"));
    }

    #[test]
    fn test_session_state_creation() {
        let session = SessionState::new();
        assert_eq!(session.current_stage, WorkflowStage::Discovery);
        assert!(session.history.records.is_empty());
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new();

        // Create a session
        let session_id = manager.create_session().unwrap();

        // Verify it exists
        let session = manager.get_session(&session_id).unwrap();
        assert!(session.is_some());

        // Record an analysis
        manager
            .record_analysis(
                &session_id,
                "test_tool".to_string(),
                serde_json::json!({}),
                true,
                None,
                vec![],
            )
            .unwrap();

        // Verify the record was added
        let updated_session = manager.get_session(&session_id).unwrap().unwrap();
        assert_eq!(updated_session.history.records.len(), 1);
    }

    #[test]
    fn test_workflow_stage_detection() {
        let mut history = AnalysisHistory::default();

        // Add some mapping stage tools
        for tool in ["search_symbols", "find_dependencies"] {
            history.add_record(AnalysisRecord::new(
                tool.to_string(),
                serde_json::json!({}),
                true,
                None,
                vec![],
            ));
        }

        let stage = history.detect_workflow_stage();
        assert_eq!(stage, WorkflowStage::Mapping);
    }
}
