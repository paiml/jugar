//! Entity-Component-System implementation
//!
//! A lightweight, cache-friendly ECS designed for WASM targets.
//! Follows Data-Oriented Design principles for optimal performance.

use core::any::{Any, TypeId};
use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{CoreError, Result};

/// Unique identifier for an entity in the world.
///
/// Entities are lightweight handles - just a generation-tagged index.
/// Components are stored separately in contiguous arrays.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(pub u64);

impl Entity {
    /// Creates a new entity with the given ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID of this entity
    #[must_use]
    pub const fn id(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// Storage for a single component type
struct ComponentStorage {
    data: HashMap<Entity, Box<dyn Any + Send + Sync>>,
}

impl ComponentStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn insert<T: Any + Send + Sync>(&mut self, entity: Entity, component: T) {
        let _ = self.data.insert(entity, Box::new(component));
    }

    fn get<T: Any>(&self, entity: Entity) -> Option<&T> {
        self.data.get(&entity).and_then(|c| c.downcast_ref())
    }

    fn get_mut<T: Any>(&mut self, entity: Entity) -> Option<&mut T> {
        self.data.get_mut(&entity).and_then(|c| c.downcast_mut())
    }

    fn remove(&mut self, entity: Entity) -> bool {
        self.data.remove(&entity).is_some()
    }

    fn contains(&self, entity: Entity) -> bool {
        self.data.contains_key(&entity)
    }
}

/// The game world containing all entities and their components.
///
/// # Example
///
/// ```
/// use jugar_core::{World, Entity, Position, Velocity};
///
/// let mut world = World::new();
/// let entity = world.spawn();
///
/// world.add_component(entity, Position::new(0.0, 0.0));
/// world.add_component(entity, Velocity::new(1.0, 0.0));
///
/// // Query and update
/// if let Some(pos) = world.get_component_mut::<Position>(entity) {
///     pos.x += 10.0;
/// }
/// ```
pub struct World {
    next_entity_id: u64,
    entities: Vec<Entity>,
    components: HashMap<TypeId, ComponentStorage>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Creates a new empty world
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }

    /// Spawns a new entity and returns its handle
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Despawns an entity and removes all its components
    ///
    /// # Errors
    ///
    /// Returns `CoreError::EntityNotFound` if the entity doesn't exist.
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        let idx = self
            .entities
            .iter()
            .position(|&e| e == entity)
            .ok_or(CoreError::EntityNotFound(entity))?;

        let _ = self.entities.swap_remove(idx);

        // Remove all components for this entity
        for storage in self.components.values_mut() {
            let _ = storage.remove(entity);
        }

        Ok(())
    }

    /// Adds a component to an entity
    ///
    /// If the entity already has this component type, it is replaced.
    pub fn add_component<T: Any + Send + Sync>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        self.components
            .entry(type_id)
            .or_insert_with(ComponentStorage::new)
            .insert(entity, component);
    }

    /// Gets a reference to a component on an entity
    #[must_use]
    pub fn get_component<T: Any>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id).and_then(|s| s.get(entity))
    }

    /// Gets a mutable reference to a component on an entity
    pub fn get_component_mut<T: Any>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|s| s.get_mut(entity))
    }

    /// Checks if an entity has a specific component
    #[must_use]
    pub fn has_component<T: Any>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .is_some_and(|s| s.contains(entity))
    }

    /// Removes a component from an entity
    ///
    /// Returns true if the component was removed, false if it didn't exist.
    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .is_some_and(|s| s.remove(entity))
    }

    /// Returns the number of entities in the world
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Returns an iterator over all entities
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }

    /// Checks if an entity exists in the world
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("entity_count", &self.entities.len())
            .field("component_types", &self.components.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Position, Velocity};

    // ==================== ENTITY TESTS ====================

    #[test]
    fn test_entity_creation() {
        let e = Entity::new(42);
        assert_eq!(e.id(), 42);
    }

    #[test]
    fn test_entity_display() {
        let e = Entity::new(123);
        assert_eq!(format!("{e}"), "e123");
    }

    #[test]
    fn test_entity_debug() {
        let e = Entity::new(456);
        assert_eq!(format!("{e:?}"), "Entity(456)");
    }

    #[test]
    fn test_entity_equality() {
        let e1 = Entity::new(1);
        let e2 = Entity::new(1);
        let e3 = Entity::new(2);
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_entity_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Entity::new(1));
        set.insert(Entity::new(2));
        set.insert(Entity::new(1)); // Duplicate
        assert_eq!(set.len(), 2);
    }

    // ==================== WORLD SPAWN/DESPAWN TESTS ====================

    #[test]
    fn test_world_spawn_increments_id() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        assert_eq!(e1.id(), 0);
        assert_eq!(e2.id(), 1);
        assert_eq!(e3.id(), 2);
    }

    #[test]
    fn test_world_entity_count() {
        let mut world = World::new();
        assert_eq!(world.entity_count(), 0);

        world.spawn();
        assert_eq!(world.entity_count(), 1);

        world.spawn();
        world.spawn();
        assert_eq!(world.entity_count(), 3);
    }

    #[test]
    fn test_world_despawn() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();

        assert_eq!(world.entity_count(), 2);
        assert!(world.despawn(e1).is_ok());
        assert_eq!(world.entity_count(), 1);
        assert!(!world.contains(e1));
        assert!(world.contains(e2));
    }

    #[test]
    fn test_world_despawn_nonexistent() {
        let mut world = World::new();
        let fake = Entity::new(999);
        let result = world.despawn(fake);
        assert!(matches!(result, Err(CoreError::EntityNotFound(_))));
    }

    #[test]
    fn test_world_contains() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.contains(e));
        assert!(!world.contains(Entity::new(999)));
    }

    // ==================== COMPONENT TESTS ====================

    #[test]
    fn test_add_and_get_component() {
        let mut world = World::new();
        let e = world.spawn();

        world.add_component(e, Position::new(10.0, 20.0));

        let pos = world.get_component::<Position>(e);
        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert!((pos.x - 10.0).abs() < f32::EPSILON);
        assert!((pos.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position::new(0.0, 0.0));

        if let Some(pos) = world.get_component_mut::<Position>(e) {
            pos.x = 100.0;
            pos.y = 200.0;
        }

        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.x - 100.0).abs() < f32::EPSILON);
        assert!((pos.y - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_has_component() {
        let mut world = World::new();
        let e = world.spawn();

        assert!(!world.has_component::<Position>(e));
        world.add_component(e, Position::new(0.0, 0.0));
        assert!(world.has_component::<Position>(e));
        assert!(!world.has_component::<Velocity>(e));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position::new(0.0, 0.0));

        assert!(world.has_component::<Position>(e));
        assert!(world.remove_component::<Position>(e));
        assert!(!world.has_component::<Position>(e));
        assert!(!world.remove_component::<Position>(e)); // Already removed
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let e = world.spawn();

        world.add_component(e, Position::new(1.0, 2.0));
        world.add_component(e, Velocity::new(3.0, 4.0));

        let pos = world.get_component::<Position>(e).unwrap();
        let vel = world.get_component::<Velocity>(e).unwrap();

        assert!((pos.x - 1.0).abs() < f32::EPSILON);
        assert!((vel.x - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_despawn_removes_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position::new(0.0, 0.0));

        world.despawn(e).unwrap();

        // Entity gone, so component lookup should fail
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn test_component_replacement() {
        let mut world = World::new();
        let e = world.spawn();

        world.add_component(e, Position::new(1.0, 1.0));
        world.add_component(e, Position::new(2.0, 2.0)); // Replace

        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.x - 2.0).abs() < f32::EPSILON);
    }

    // ==================== ITERATOR TESTS ====================

    #[test]
    fn test_entities_iterator() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        let entities: Vec<_> = world.entities().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    // ==================== BEHAVIORAL TESTS (MUTATION-RESISTANT) ====================

    #[test]
    fn test_position_actually_moves_after_velocity_applied() {
        let mut world = World::new();
        let e = world.spawn();

        world.add_component(e, Position::new(0.0, 0.0));
        world.add_component(e, Velocity::new(10.0, 5.0));

        // Simulate one physics step
        let dt = 1.0;
        let vel = *world.get_component::<Velocity>(e).unwrap();
        if let Some(pos) = world.get_component_mut::<Position>(e) {
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
        }

        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.x - 10.0).abs() < f32::EPSILON, "X should move by velocity");
        assert!((pos.y - 5.0).abs() < f32::EPSILON, "Y should move by velocity");
    }
}
