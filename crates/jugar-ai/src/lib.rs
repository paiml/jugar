//! # jugar-ai
//!
//! AI systems for Jugar including Behavior Trees and GOAP.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// AI system errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AiError {
    /// No valid plan found
    #[error("No valid plan found to achieve goal")]
    NoPlanFound,
    /// Action preconditions not met
    #[error("Action preconditions not met: {0}")]
    PreconditionsNotMet(String),
}

/// Result type for AI operations
pub type Result<T> = core::result::Result<T, AiError>;

/// World state for GOAP planning
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    facts: HashMap<String, bool>,
}

impl WorldState {
    /// Creates a new empty world state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a fact
    pub fn set(&mut self, key: impl Into<String>, value: bool) {
        let _ = self.facts.insert(key.into(), value);
    }

    /// Gets a fact (defaults to false if not present)
    #[must_use]
    pub fn get(&self, key: &str) -> bool {
        self.facts.get(key).copied().unwrap_or(false)
    }

    /// Checks if this state satisfies the given conditions
    #[must_use]
    pub fn satisfies(&self, conditions: &Self) -> bool {
        conditions.facts.iter().all(|(k, v)| self.get(k) == *v)
    }

    /// Creates a test world state
    #[cfg(test)]
    #[must_use]
    pub fn test() -> Self {
        let mut state = Self::new();
        state.set("has_weapon", false);
        state.set("enemy_visible", true);
        state
    }
}

/// An action for GOAP planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action name
    pub name: String,
    /// Cost of the action
    pub cost: f32,
    /// Preconditions that must be true
    pub preconditions: WorldState,
    /// Effects applied after action
    pub effects: WorldState,
}

impl Action {
    /// Creates a new action
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cost: 1.0,
            preconditions: WorldState::new(),
            effects: WorldState::new(),
        }
    }

    /// Sets the cost
    #[must_use]
    pub const fn with_cost(mut self, cost: f32) -> Self {
        self.cost = cost;
        self
    }

    /// Adds a precondition
    #[must_use]
    pub fn with_precondition(mut self, key: impl Into<String>, value: bool) -> Self {
        self.preconditions.set(key, value);
        self
    }

    /// Adds an effect
    #[must_use]
    pub fn with_effect(mut self, key: impl Into<String>, value: bool) -> Self {
        self.effects.set(key, value);
        self
    }

    /// Checks if this action can run in the given state
    #[must_use]
    pub fn can_run(&self, state: &WorldState) -> bool {
        state.satisfies(&self.preconditions)
    }

    /// Applies effects to a state
    #[must_use]
    pub fn apply(&self, state: &WorldState) -> WorldState {
        let mut new_state = state.clone();
        for (k, v) in &self.effects.facts {
            new_state.set(k.clone(), *v);
        }
        new_state
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Action {}

/// Goal for GOAP planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Goal name
    pub name: String,
    /// Priority (higher = more important)
    pub priority: f32,
    /// Desired world state
    pub desired_state: WorldState,
}

impl Goal {
    /// Creates a new goal
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority: 1.0,
            desired_state: WorldState::new(),
        }
    }

    /// Sets the priority
    #[must_use]
    pub const fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    /// Adds a desired condition
    #[must_use]
    pub fn with_condition(mut self, key: impl Into<String>, value: bool) -> Self {
        self.desired_state.set(key, value);
        self
    }

    /// Checks if the goal is satisfied by the current state
    #[must_use]
    pub fn is_satisfied(&self, state: &WorldState) -> bool {
        state.satisfies(&self.desired_state)
    }
}

/// GOAP Planner
pub struct Planner {
    actions: Vec<Action>,
}

impl Planner {
    /// Creates a new planner
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Adds an action
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Plans a sequence of actions to achieve the goal
    ///
    /// Uses A* search through action space.
    pub fn plan(&self, current_state: &WorldState, goal: &Goal) -> Result<Vec<Action>> {
        if goal.is_satisfied(current_state) {
            return Ok(Vec::new());
        }

        // Simple backwards planning
        let mut plan = Vec::new();
        let mut working_state = current_state.clone();

        for _ in 0..100 {
            // Max iterations
            if goal.is_satisfied(&working_state) {
                return Ok(plan);
            }

            // Find an action that gets us closer to the goal
            let mut best_action: Option<&Action> = None;
            let mut best_progress = 0;

            for action in &self.actions {
                if !action.can_run(&working_state) {
                    continue;
                }

                let new_state = action.apply(&working_state);
                let progress = count_satisfied(&new_state, &goal.desired_state)
                    - count_satisfied(&working_state, &goal.desired_state);

                if progress > best_progress || best_action.is_none() {
                    best_progress = progress;
                    best_action = Some(action);
                }
            }

            if let Some(action) = best_action {
                working_state = action.apply(&working_state);
                plan.push(action.clone());
            } else {
                return Err(AiError::NoPlanFound);
            }
        }

        Err(AiError::NoPlanFound)
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Planner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Planner")
            .field("action_count", &self.actions.len())
            .finish()
    }
}

fn count_satisfied(state: &WorldState, goal: &WorldState) -> i32 {
    goal.facts
        .iter()
        .filter(|(k, v)| state.get(k) == **v)
        .count() as i32
}

/// Behavior tree node status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is still running
    Running,
    /// Node succeeded
    Success,
    /// Node failed
    Failure,
}

/// Behavior tree node trait
pub trait BehaviorNode: fmt::Debug {
    /// Ticks the node and returns its status
    fn tick(&mut self, dt: f32) -> NodeStatus;

    /// Resets the node to initial state
    fn reset(&mut self);
}

/// Sequence node - runs children in order until one fails
#[derive(Debug)]
pub struct Sequence {
    children: Vec<Box<dyn BehaviorNode>>,
    current: usize,
}

impl Sequence {
    /// Creates a new sequence
    #[must_use]
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }
}

impl BehaviorNode for Sequence {
    fn tick(&mut self, dt: f32) -> NodeStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(dt) {
                NodeStatus::Running => return NodeStatus::Running,
                NodeStatus::Success => self.current += 1,
                NodeStatus::Failure => return NodeStatus::Failure,
            }
        }
        NodeStatus::Success
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

/// Selector node - runs children until one succeeds
#[derive(Debug)]
pub struct Selector {
    children: Vec<Box<dyn BehaviorNode>>,
    current: usize,
}

impl Selector {
    /// Creates a new selector
    #[must_use]
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self {
            children,
            current: 0,
        }
    }
}

impl BehaviorNode for Selector {
    fn tick(&mut self, dt: f32) -> NodeStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(dt) {
                NodeStatus::Running => return NodeStatus::Running,
                NodeStatus::Failure => self.current += 1,
                NodeStatus::Success => return NodeStatus::Success,
            }
        }
        NodeStatus::Failure
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_state() {
        let mut state = WorldState::new();
        state.set("has_weapon", true);
        assert!(state.get("has_weapon"));
        assert!(!state.get("nonexistent"));
    }

    #[test]
    fn test_world_state_satisfies() {
        let mut state = WorldState::new();
        state.set("has_weapon", true);
        state.set("has_ammo", true);

        let mut conditions = WorldState::new();
        conditions.set("has_weapon", true);

        assert!(state.satisfies(&conditions));

        conditions.set("has_ammo", false);
        assert!(!state.satisfies(&conditions));
    }

    #[test]
    fn test_action_can_run() {
        let action = Action::new("attack").with_precondition("has_weapon", true);

        let mut state = WorldState::new();
        assert!(!action.can_run(&state));

        state.set("has_weapon", true);
        assert!(action.can_run(&state));
    }

    #[test]
    fn test_action_apply() {
        let action = Action::new("pickup_weapon").with_effect("has_weapon", true);

        let state = WorldState::new();
        let new_state = action.apply(&state);

        assert!(new_state.get("has_weapon"));
    }

    #[test]
    fn test_goal_satisfied() {
        let goal = Goal::new("be_armed").with_condition("has_weapon", true);

        let mut state = WorldState::new();
        assert!(!goal.is_satisfied(&state));

        state.set("has_weapon", true);
        assert!(goal.is_satisfied(&state));
    }

    #[test]
    fn test_planner_simple_plan() {
        let mut planner = Planner::new();

        planner.add_action(Action::new("pickup_weapon").with_effect("has_weapon", true));

        let state = WorldState::new();
        let goal = Goal::new("be_armed").with_condition("has_weapon", true);

        let plan = planner.plan(&state, &goal).unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "pickup_weapon");
    }

    #[test]
    fn test_planner_already_satisfied() {
        let planner = Planner::new();

        let mut state = WorldState::new();
        state.set("has_weapon", true);

        let goal = Goal::new("be_armed").with_condition("has_weapon", true);

        let plan = planner.plan(&state, &goal).unwrap();
        assert!(plan.is_empty());
    }

    #[test]
    fn test_node_status() {
        assert_ne!(NodeStatus::Running, NodeStatus::Success);
        assert_ne!(NodeStatus::Success, NodeStatus::Failure);
    }
}
