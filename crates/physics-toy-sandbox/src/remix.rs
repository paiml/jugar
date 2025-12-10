//! Remix Graph - Lineage tracking for contraptions
//!
//! Implements Git-like content-addressable storage for the remix system:
//! - Fork/edit/share workflow (Kaizen cycle)
//! - Attribution and lineage tracking
//! - Quality assurance for user-generated content

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Contraption, ContraptionId, Result, SandboxError};

/// Remix graph for tracking contraption lineage
///
/// Implements the Kaizen cycle: DISCOVER → FORK → EDIT → SHARE
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemixGraph {
    /// Map from contraption ID to its parent
    lineage: HashMap<ContraptionId, ContraptionId>,

    /// Depth of remix chain (root = 0)
    depth: HashMap<ContraptionId, u32>,

    /// Child count for each contraption
    children_count: HashMap<ContraptionId, u32>,
}

impl RemixGraph {
    /// Create a new empty remix graph
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new contraption in the graph
    pub fn register(&mut self, contraption: &Contraption) {
        if let Some(parent_id) = contraption.forked_from {
            // This is a fork
            let _ = self.lineage.insert(contraption.id, parent_id);

            // Calculate depth
            let parent_depth = self.depth.get(&parent_id).copied().unwrap_or(0);
            let _ = self.depth.insert(contraption.id, parent_depth + 1);

            // Increment parent's child count
            *self.children_count.entry(parent_id).or_insert(0) += 1;
        } else {
            // This is a root contraption
            let _ = self.depth.insert(contraption.id, 0);
        }
    }

    /// Get the parent of a contraption
    #[must_use]
    pub fn parent(&self, id: ContraptionId) -> Option<ContraptionId> {
        self.lineage.get(&id).copied()
    }

    /// Get full ancestry back to original (root)
    #[must_use]
    pub fn ancestors(&self, id: ContraptionId) -> Vec<ContraptionId> {
        let mut ancestors = Vec::new();
        let mut current = id;

        while let Some(parent) = self.lineage.get(&current) {
            ancestors.push(*parent);
            current = *parent;
        }

        ancestors
    }

    /// Get the root (original) contraption of a remix chain
    #[must_use]
    pub fn root(&self, id: ContraptionId) -> ContraptionId {
        let mut current = id;

        while let Some(parent) = self.lineage.get(&current) {
            current = *parent;
        }

        current
    }

    /// Get depth of a contraption in the remix chain (0 = original)
    #[must_use]
    pub fn depth(&self, id: ContraptionId) -> u32 {
        self.depth.get(&id).copied().unwrap_or(0)
    }

    /// Count total remixes descended from this contraption
    #[must_use]
    pub fn descendant_count(&self, id: ContraptionId) -> u32 {
        self.children_count.get(&id).copied().unwrap_or(0)
    }

    /// Check if `potential_ancestor` is an ancestor of `id`
    #[must_use]
    pub fn is_ancestor_of(&self, potential_ancestor: ContraptionId, id: ContraptionId) -> bool {
        self.ancestors(id).contains(&potential_ancestor)
    }

    /// Check if a contraption is a root (not a fork)
    #[must_use]
    pub fn is_root(&self, id: ContraptionId) -> bool {
        !self.lineage.contains_key(&id)
    }

    /// Get all root contraptions
    #[must_use]
    pub fn roots(&self) -> Vec<ContraptionId> {
        self.depth
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get all children (direct forks) of a contraption
    #[must_use]
    pub fn children(&self, parent_id: ContraptionId) -> Vec<ContraptionId> {
        self.lineage
            .iter()
            .filter(|(_, &p)| p == parent_id)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get statistics about the graph
    #[must_use]
    pub fn stats(&self) -> RemixStats {
        let total = self.depth.len();
        let roots = self.depth.values().filter(|&&d| d == 0).count();
        let forks = total - roots;
        let max_depth = self.depth.values().max().copied().unwrap_or(0);

        RemixStats {
            total_contraptions: total,
            root_contraptions: roots,
            fork_count: forks,
            max_chain_depth: max_depth,
        }
    }

    /// Remove a contraption from the graph
    ///
    /// Note: This orphans any children (they become roots)
    pub fn remove(&mut self, id: ContraptionId) {
        // Remove from lineage
        if let Some(parent) = self.lineage.remove(&id) {
            // Decrement parent's child count
            if let Some(count) = self.children_count.get_mut(&parent) {
                *count = count.saturating_sub(1);
            }
        }

        // Remove depth entry
        let _ = self.depth.remove(&id);

        // Orphan any children (make them roots)
        let children: Vec<_> = self
            .lineage
            .iter()
            .filter(|(_, &p)| p == id)
            .map(|(&c, _)| c)
            .collect();

        for child in children {
            let _ = self.lineage.remove(&child);
            let _ = self.depth.insert(child, 0);
        }

        // Remove child count
        let _ = self.children_count.remove(&id);
    }
}

/// Statistics about the remix graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemixStats {
    /// Total number of contraptions
    pub total_contraptions: usize,

    /// Number of root (original) contraptions
    pub root_contraptions: usize,

    /// Number of forked contraptions
    pub fork_count: usize,

    /// Maximum depth of remix chains
    pub max_chain_depth: u32,
}

/// Storage for contraptions with remix tracking
#[derive(Debug, Clone, Default)]
pub struct ContraptionStorage {
    /// All stored contraptions
    contraptions: HashMap<ContraptionId, Contraption>,

    /// Remix graph for lineage tracking
    graph: RemixGraph,

    /// Content hash index for deduplication
    content_hashes: HashMap<u32, ContraptionId>,
}

impl ContraptionStorage {
    /// Create a new empty storage
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Save a contraption
    ///
    /// # Errors
    /// Returns error if validation fails
    pub fn save(&mut self, contraption: Contraption) -> Result<ContraptionId> {
        contraption.validate()?;

        let id = contraption.id;
        let hash = contraption.content_hash();

        // Register in graph
        self.graph.register(&contraption);

        // Store contraption
        let _ = self.contraptions.insert(id, contraption);

        // Index by content hash
        let _ = self.content_hashes.insert(hash, id);

        Ok(id)
    }

    /// Load a contraption by ID
    ///
    /// # Errors
    /// Returns error if not found
    pub fn load(&self, id: ContraptionId) -> Result<&Contraption> {
        self.contraptions.get(&id).ok_or(SandboxError::NotFound(id))
    }

    /// Check if a contraption exists
    #[must_use]
    pub fn exists(&self, id: ContraptionId) -> bool {
        self.contraptions.contains_key(&id)
    }

    /// Find contraption by content hash (deduplication)
    #[must_use]
    pub fn find_by_hash(&self, hash: u32) -> Option<ContraptionId> {
        self.content_hashes.get(&hash).copied()
    }

    /// Get the remix graph
    #[must_use]
    pub const fn graph(&self) -> &RemixGraph {
        &self.graph
    }

    /// Get mutable reference to remix graph
    pub fn graph_mut(&mut self) -> &mut RemixGraph {
        &mut self.graph
    }

    /// Delete a contraption
    pub fn delete(&mut self, id: ContraptionId) -> Option<Contraption> {
        if let Some(contraption) = self.contraptions.remove(&id) {
            let hash = contraption.content_hash();
            let _ = self.content_hashes.remove(&hash);
            self.graph.remove(id);
            Some(contraption)
        } else {
            None
        }
    }

    /// Get all contraptions
    #[must_use]
    pub fn all(&self) -> Vec<&Contraption> {
        self.contraptions.values().collect()
    }

    /// Get count of stored contraptions
    #[must_use]
    pub fn count(&self) -> usize {
        self.contraptions.len()
    }

    /// Search by tag
    #[must_use]
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Contraption> {
        self.contraptions
            .values()
            .filter(|c| c.metadata.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Get popular contraptions (by remix count)
    #[must_use]
    pub fn popular(&self, limit: usize) -> Vec<&Contraption> {
        let mut sorted: Vec<_> = self.contraptions.values().collect();
        sorted.sort_by(|a, b| b.metadata.remix_count.cmp(&a.metadata.remix_count));
        sorted.truncate(limit);
        sorted
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::similar_names,
    unused_results
)]
mod tests {
    use super::*;
    use crate::ContraptionBuilder;
    use crate::ObjectType;
    use crate::Transform2D;

    // =========================================================================
    // EXTREME TDD: Remix system tests from specification
    // =========================================================================

    mod remix_graph_tests {
        use super::*;

        #[test]
        fn test_register_root_contraption() {
            let mut graph = RemixGraph::new();
            let c = Contraption::new("Root");
            graph.register(&c);

            assert!(graph.is_root(c.id));
            assert_eq!(graph.depth(c.id), 0);
        }

        #[test]
        fn test_register_fork() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork = root.fork("Fork");
            graph.register(&fork);

            assert!(!graph.is_root(fork.id));
            assert_eq!(graph.depth(fork.id), 1);
            assert_eq!(graph.parent(fork.id), Some(root.id));
        }

        #[test]
        fn test_ancestors() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork1 = root.fork("Fork 1");
            graph.register(&fork1);

            let fork2 = fork1.fork("Fork 2");
            graph.register(&fork2);

            let ancestors = graph.ancestors(fork2.id);
            assert_eq!(ancestors.len(), 2);
            assert_eq!(ancestors[0], fork1.id);
            assert_eq!(ancestors[1], root.id);
        }

        #[test]
        fn test_root_function() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork1 = root.fork("Fork 1");
            graph.register(&fork1);

            let fork2 = fork1.fork("Fork 2");
            graph.register(&fork2);

            assert_eq!(graph.root(fork2.id), root.id);
            assert_eq!(graph.root(fork1.id), root.id);
            assert_eq!(graph.root(root.id), root.id);
        }

        #[test]
        fn test_descendant_count() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork1 = root.fork("Fork 1");
            graph.register(&fork1);

            let fork2 = root.fork("Fork 2");
            graph.register(&fork2);

            assert_eq!(graph.descendant_count(root.id), 2);
        }

        #[test]
        fn test_is_ancestor_of() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork = root.fork("Fork");
            graph.register(&fork);

            assert!(graph.is_ancestor_of(root.id, fork.id));
            assert!(!graph.is_ancestor_of(fork.id, root.id));
        }

        #[test]
        fn test_children() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork1 = root.fork("Fork 1");
            graph.register(&fork1);

            let fork2 = root.fork("Fork 2");
            graph.register(&fork2);

            let children = graph.children(root.id);
            assert_eq!(children.len(), 2);
            assert!(children.contains(&fork1.id));
            assert!(children.contains(&fork2.id));
        }

        #[test]
        fn test_stats() {
            let mut graph = RemixGraph::new();

            let root1 = Contraption::new("Root 1");
            graph.register(&root1);

            let root2 = Contraption::new("Root 2");
            graph.register(&root2);

            let fork1 = root1.fork("Fork 1");
            graph.register(&fork1);

            let fork2 = fork1.fork("Fork 2");
            graph.register(&fork2);

            let stats = graph.stats();
            assert_eq!(stats.total_contraptions, 4);
            assert_eq!(stats.root_contraptions, 2);
            assert_eq!(stats.fork_count, 2);
            assert_eq!(stats.max_chain_depth, 2);
        }

        #[test]
        fn test_remove_orphans_children() {
            let mut graph = RemixGraph::new();

            let root = Contraption::new("Root");
            graph.register(&root);

            let fork = root.fork("Fork");
            graph.register(&fork);

            // Remove root - fork should become a root
            graph.remove(root.id);

            assert!(graph.is_root(fork.id));
            assert_eq!(graph.depth(fork.id), 0);
        }
    }

    mod storage_tests {
        use super::*;

        #[test]
        fn test_save_and_load() {
            let mut storage = ContraptionStorage::new();

            let contraption = ContraptionBuilder::new("Test")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            let id = storage.save(contraption.clone()).unwrap();
            let loaded = storage.load(id).unwrap();

            assert_eq!(loaded.id, contraption.id);
        }

        #[test]
        fn test_load_not_found() {
            let storage = ContraptionStorage::new();
            let fake_id = ContraptionId::new();
            let result = storage.load(fake_id);
            assert!(matches!(result, Err(SandboxError::NotFound(_))));
        }

        #[test]
        fn test_deduplication_by_hash() {
            let mut storage = ContraptionStorage::new();

            let c1 = ContraptionBuilder::new("Scene A")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            let hash = c1.content_hash();
            storage.save(c1.clone()).unwrap();

            // Same content hash should find the existing contraption
            let found = storage.find_by_hash(hash);
            assert_eq!(found, Some(c1.id));
        }

        #[test]
        fn test_delete() {
            let mut storage = ContraptionStorage::new();

            let contraption = Contraption::new("Test");
            let id = contraption.id;
            storage.save(contraption).unwrap();

            assert!(storage.exists(id));
            storage.delete(id);
            assert!(!storage.exists(id));
        }

        #[test]
        fn test_search_by_tag() {
            let mut storage = ContraptionStorage::new();

            let c1 = ContraptionBuilder::new("Physics Demo")
                .tag("physics")
                .tag("demo")
                .build()
                .unwrap();

            let c2 = ContraptionBuilder::new("Art Project")
                .tag("art")
                .build()
                .unwrap();

            storage.save(c1).unwrap();
            storage.save(c2).unwrap();

            let results = storage.search_by_tag("physics");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].metadata.name, "Physics Demo");
        }

        #[test]
        fn test_popular() {
            let mut storage = ContraptionStorage::new();

            let mut c1 = Contraption::new("Popular");
            c1.metadata.remix_count = 100;
            storage.save(c1).unwrap();

            let mut c2 = Contraption::new("Less Popular");
            c2.metadata.remix_count = 10;
            storage.save(c2).unwrap();

            let popular = storage.popular(1);
            assert_eq!(popular.len(), 1);
            assert_eq!(popular[0].metadata.name, "Popular");
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_remix_workflow() {
            let mut storage = ContraptionStorage::new();

            // 1. Create original
            let original = ContraptionBuilder::new("Original Machine")
                .author("Creator")
                .with_object(ObjectType::Ball, Transform2D::default())
                .with_object(
                    ObjectType::Ramp,
                    Transform2D {
                        position: glam::Vec2::new(100.0, 0.0),
                        ..Transform2D::default()
                    },
                )
                .build()
                .unwrap();

            let original_id = storage.save(original).unwrap();

            // 2. Fork it
            let original_ref = storage.load(original_id).unwrap();
            let forked = original_ref.fork("Remixed Machine");
            let fork_id = storage.save(forked).unwrap();

            // 3. Verify lineage
            assert_eq!(storage.graph().parent(fork_id), Some(original_id));
            assert!(storage.graph().is_ancestor_of(original_id, fork_id));

            // 4. Fork the fork
            let fork_ref = storage.load(fork_id).unwrap();
            let forked2 = fork_ref.fork("Twice Remixed");
            let fork2_id = storage.save(forked2).unwrap();

            // 5. Verify full ancestry
            let ancestors = storage.graph().ancestors(fork2_id);
            assert_eq!(ancestors.len(), 2);
            assert_eq!(storage.graph().root(fork2_id), original_id);
            assert_eq!(storage.graph().depth(fork2_id), 2);
        }
    }
}
