// ═══════════════════════════════════════════════════════════════════════════════
// API handlers for diagram CRUD operations
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{error::Result, AppState, DiagramData};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory diagram storage (TODO: replace with persistent storage)
pub type DiagramStore = Arc<RwLock<HashMap<String, DiagramData>>>;

impl DiagramApi {
    /// Create a new DiagramApi with the given store
    pub fn new(store: DiagramStore) -> Self {
        Self { store }
    }

    /// Create with empty in-memory store
    pub fn with_memory_store() -> Self {
        Self::new(Arc::new(RwLock::new(HashMap::new())))
    }
}

/// API for diagram operations
#[derive(Clone)]
pub struct DiagramApi {
    pub store: DiagramStore,
}

impl DiagramApi {
    /// List all diagrams
    pub async fn list_diagrams(&self) -> Result<Vec<DiagramData>> {
        let store = self.store.read().await;
        Ok(store.values().cloned().collect())
    }

    /// Get a specific diagram by ID
    pub async fn get_diagram(&self, id: &str) -> Result<DiagramData> {
        let store = self.store.read().await;
        store
            .get(id)
            .cloned()
            .ok_or_else(|| crate::error::Error::DiagramNotFound(id.to_string()))
    }

    /// Create a new diagram
    pub async fn create_diagram(&self, diagram: DiagramData) -> Result<DiagramData> {
        let mut store = self.store.write().await;
        let id = diagram.id.clone();
        store.insert(id.clone(), diagram.clone());
        Ok(diagram)
    }

    /// Update an existing diagram
    pub async fn update_diagram(&self, id: &str, diagram: DiagramData) -> Result<DiagramData> {
        let mut store = self.store.write().await;
        if !store.contains_key(id) {
            return Err(crate::error::Error::DiagramNotFound(id.to_string()));
        }
        store.insert(id.to_string(), diagram.clone());
        Ok(diagram)
    }

    /// Delete a diagram
    pub async fn delete_diagram(&self, id: &str) -> Result<()> {
        let mut store = self.store.write().await;
        store
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| crate::error::Error::DiagramNotFound(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_list_diagrams() {
        let api = DiagramApi::with_memory_store();

        let diagram = DiagramData {
            id: "test-1".to_string(),
            name: "Test Diagram".to_string(),
            entities: vec![],
            connections: vec![],
        };

        api.create_diagram(diagram.clone()).await.unwrap();

        let diagrams = api.list_diagrams().await.unwrap();
        assert_eq!(diagrams.len(), 1);
        assert_eq!(diagrams[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_get_diagram() {
        let api = DiagramApi::with_memory_store();

        let diagram = DiagramData {
            id: "test-2".to_string(),
            name: "Test Diagram 2".to_string(),
            entities: vec![],
            connections: vec![],
        };

        api.create_diagram(diagram.clone()).await.unwrap();

        let retrieved = api.get_diagram("test-2").await.unwrap();
        assert_eq!(retrieved.name, "Test Diagram 2");
    }

    #[tokio::test]
    async fn test_get_nonexistent_diagram() {
        let api = DiagramApi::with_memory_store();

        let result = api.get_diagram("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_diagram() {
        let api = DiagramApi::with_memory_store();

        let diagram = DiagramData {
            id: "test-3".to_string(),
            name: "Test Diagram 3".to_string(),
            entities: vec![],
            connections: vec![],
        };

        api.create_diagram(diagram).await.unwrap();
        api.delete_diagram("test-3").await.unwrap();

        let diagrams = api.list_diagrams().await.unwrap();
        assert_eq!(diagrams.len(), 0);
    }
}
