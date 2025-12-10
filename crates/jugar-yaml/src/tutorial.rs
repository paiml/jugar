//! Tutorial progression system.
//!
//! Per spec Section 10.1: Staged tutorial for introducing game concepts.

use crate::error::YamlError;
use crate::schema::SchemaLevel;

/// Tutorial stage representing progression through learning
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TutorialStage {
    /// Stage 1: Hello World - Just a character (30 seconds)
    HelloWorld = 1,
    /// Stage 2: Add a Goal - Character + collectible (60 seconds)
    AddGoal = 2,
    /// Stage 3: Add Feedback - Events and responses (90 seconds)
    AddFeedback = 3,
    /// Stage 4: Make It Challenging - Full game mechanics (2 minutes)
    MakeChallenging = 4,
}

impl TutorialStage {
    /// Get the stage number (1-4)
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// Get human-readable stage name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HelloWorld => "Hello World",
            Self::AddGoal => "Add a Goal",
            Self::AddFeedback => "Add Feedback",
            Self::MakeChallenging => "Make It Challenging",
        }
    }

    /// Get the estimated time to complete this stage
    #[must_use]
    pub const fn estimated_seconds(self) -> u32 {
        match self {
            Self::HelloWorld => 30,
            Self::AddGoal => 60,
            Self::AddFeedback => 90,
            Self::MakeChallenging => 120,
        }
    }

    /// Get example YAML for this stage
    #[must_use]
    pub const fn example_yaml(self) -> &'static str {
        match self {
            Self::HelloWorld => "character: bunny",
            Self::AddGoal => "character: bunny\ncollect: stars",
            Self::AddFeedback => "character: bunny\ncollect: stars\nwhen_collect:\n  sound: twinkle\n  score: +1",
            Self::MakeChallenging => "character: bunny\ncollect: stars\navoid: spiders\nwhen_collect:\n  sound: twinkle\n  score: +1\nwhen_avoid:\n  sound: oops\n  lives: -1\nlives: 3",
        }
    }

    /// Get instructions for this stage (kid-friendly)
    #[must_use]
    pub const fn instructions(self) -> &'static str {
        match self {
            Self::HelloWorld => "Type 'character: bunny' to make a bunny appear! It will follow your finger or mouse.",
            Self::AddGoal => "Add 'collect: stars' on a new line. Now your bunny can catch stars!",
            Self::AddFeedback => "Add 'when_collect:' with 'sound: twinkle' and 'score: +1' to make it more fun!",
            Self::MakeChallenging => "Add 'avoid: spiders' and 'lives: 3' to make it a real game!",
        }
    }

    /// Get the next stage, or None if this is the last
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::HelloWorld => Some(Self::AddGoal),
            Self::AddGoal => Some(Self::AddFeedback),
            Self::AddFeedback => Some(Self::MakeChallenging),
            Self::MakeChallenging => None,
        }
    }

    /// Get all stages in order
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::HelloWorld,
            Self::AddGoal,
            Self::AddFeedback,
            Self::MakeChallenging,
        ]
    }

    /// Get the schema level appropriate for this tutorial stage
    #[must_use]
    pub const fn schema_level(self) -> SchemaLevel {
        // All tutorial stages use Level 1 (ages 5-7)
        SchemaLevel::Level1
    }
}

/// Tutorial progress tracker
#[derive(Debug, Clone)]
pub struct TutorialProgress {
    /// Current stage
    pub current_stage: TutorialStage,
    /// Stages completed
    pub completed_stages: Vec<TutorialStage>,
    /// Current YAML being edited
    pub current_yaml: String,
    /// Hints shown for current stage
    pub hints_shown: u8,
}

impl Default for TutorialProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl TutorialProgress {
    /// Create new tutorial progress at stage 1
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_stage: TutorialStage::HelloWorld,
            completed_stages: Vec::new(),
            current_yaml: String::new(),
            hints_shown: 0,
        }
    }

    /// Check if the current YAML satisfies the current stage requirements
    #[must_use]
    pub fn check_stage_completion(&self) -> StageCheckResult {
        check_yaml_for_stage(&self.current_yaml, self.current_stage)
    }

    /// Advance to next stage if current is complete
    ///
    /// # Errors
    ///
    /// Returns error if current stage is not complete
    pub fn advance(&mut self) -> Result<Option<TutorialStage>, TutorialError> {
        let check = self.check_stage_completion();
        if !check.complete {
            return Err(TutorialError::StageNotComplete {
                stage: self.current_stage,
                missing: check.missing_elements,
            });
        }

        self.completed_stages.push(self.current_stage);
        self.hints_shown = 0;

        if let Some(next) = self.current_stage.next() {
            self.current_stage = next;
            Ok(Some(next))
        } else {
            Ok(None) // Tutorial complete!
        }
    }

    /// Update the current YAML
    pub fn update_yaml(&mut self, yaml: impl Into<String>) {
        self.current_yaml = yaml.into();
    }

    /// Get a hint for the current stage
    #[must_use]
    pub fn get_hint(&mut self) -> Option<&'static str> {
        let hints = get_hints_for_stage(self.current_stage);
        if (self.hints_shown as usize) < hints.len() {
            let hint = hints[self.hints_shown as usize];
            self.hints_shown += 1;
            Some(hint)
        } else {
            None
        }
    }

    /// Check if tutorial is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.completed_stages.len() == 4
    }

    /// Get total progress percentage (0-100)
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        let completed = self.completed_stages.len();
        let current_progress = usize::from(self.check_stage_completion().complete);
        #[allow(clippy::cast_possible_truncation)]
        let percent = ((completed * 25) + (current_progress * 25)) as u8;
        percent.min(100)
    }
}

/// Result of checking YAML against stage requirements
#[derive(Debug, Clone)]
pub struct StageCheckResult {
    /// Whether the stage is complete
    pub complete: bool,
    /// Elements present in the YAML
    pub present_elements: Vec<String>,
    /// Elements missing for this stage
    pub missing_elements: Vec<String>,
    /// Helpful message for the user
    pub message: String,
}

/// Check if YAML satisfies a tutorial stage
#[must_use]
pub fn check_yaml_for_stage(yaml: &str, stage: TutorialStage) -> StageCheckResult {
    let mut present = Vec::new();
    let mut missing = Vec::new();

    // Parse YAML to check elements
    let has_character = yaml.contains("character:");
    let has_collect = yaml.contains("collect:");
    let has_when_collect = yaml.contains("when_collect:");
    let has_avoid = yaml.contains("avoid:");
    let has_lives = yaml.contains("lives:");

    if has_character {
        present.push("character".to_string());
    }
    if has_collect {
        present.push("collect".to_string());
    }
    if has_when_collect {
        present.push("when_collect".to_string());
    }
    if has_avoid {
        present.push("avoid".to_string());
    }
    if has_lives {
        present.push("lives".to_string());
    }

    match stage {
        TutorialStage::HelloWorld => {
            if !has_character {
                missing.push("character".to_string());
            }
        }
        TutorialStage::AddGoal => {
            if !has_character {
                missing.push("character".to_string());
            }
            if !has_collect {
                missing.push("collect".to_string());
            }
        }
        TutorialStage::AddFeedback => {
            if !has_character {
                missing.push("character".to_string());
            }
            if !has_collect {
                missing.push("collect".to_string());
            }
            if !has_when_collect {
                missing.push("when_collect".to_string());
            }
        }
        TutorialStage::MakeChallenging => {
            if !has_character {
                missing.push("character".to_string());
            }
            if !has_collect {
                missing.push("collect".to_string());
            }
            if !has_when_collect {
                missing.push("when_collect".to_string());
            }
            if !has_avoid {
                missing.push("avoid".to_string());
            }
            if !has_lives {
                missing.push("lives".to_string());
            }
        }
    }

    let complete = missing.is_empty();
    let message = if complete {
        format!("Great job! You completed {} ⭐", stage.name())
    } else {
        format!("Almost there! Add: {}", missing.join(", "))
    };

    StageCheckResult {
        complete,
        present_elements: present,
        missing_elements: missing,
        message,
    }
}

/// Get hints for a tutorial stage
const fn get_hints_for_stage(stage: TutorialStage) -> &'static [&'static str] {
    match stage {
        TutorialStage::HelloWorld => &[
            "Start by typing 'character:' followed by a space",
            "Try typing 'bunny' after 'character: '",
            "The full line should be 'character: bunny'",
        ],
        TutorialStage::AddGoal => &[
            "Press Enter to make a new line",
            "Type 'collect:' followed by what to collect",
            "Try 'collect: stars' - stars are fun to catch!",
        ],
        TutorialStage::AddFeedback => &[
            "Add 'when_collect:' on a new line",
            "Under 'when_collect:', indent and add 'sound: twinkle'",
            "Also add 'score: +1' to track points!",
        ],
        TutorialStage::MakeChallenging => &[
            "Add something to avoid with 'avoid: spiders'",
            "Add 'when_avoid:' with 'lives: -1' to lose lives",
            "Set starting lives with 'lives: 3' at the top",
        ],
    }
}

/// Tutorial-related errors
#[derive(Debug, Clone)]
pub enum TutorialError {
    /// Stage requirements not met
    StageNotComplete {
        /// The stage that wasn't complete
        stage: TutorialStage,
        /// What's missing
        missing: Vec<String>,
    },
    /// Invalid YAML syntax
    InvalidYaml {
        /// The error message
        message: String,
    },
}

impl core::fmt::Display for TutorialError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StageNotComplete { stage, missing } => {
                write!(
                    f,
                    "Stage '{}' not complete. Missing: {}",
                    stage.name(),
                    missing.join(", ")
                )
            }
            Self::InvalidYaml { message } => {
                write!(f, "Oops! There's a problem with your code: {message}")
            }
        }
    }
}

impl core::error::Error for TutorialError {}

impl From<TutorialError> for YamlError {
    fn from(err: TutorialError) -> Self {
        Self::ValidationError {
            message: err.to_string(),
        }
    }
}

/// Template definition for remixable games
#[derive(Debug, Clone)]
pub struct GameTemplate {
    /// Template identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description for kids
    pub description: String,
    /// Schema level (1-3)
    pub level: SchemaLevel,
    /// Preview image path
    pub preview_path: Option<String>,
    /// The YAML template content
    pub yaml: String,
}

impl GameTemplate {
    /// Create a new template
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, level: SchemaLevel) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            level,
            preview_path: None,
            yaml: String::new(),
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set YAML content
    #[must_use]
    pub fn with_yaml(mut self, yaml: impl Into<String>) -> Self {
        self.yaml = yaml.into();
        self
    }

    /// Set preview path
    #[must_use]
    pub fn with_preview(mut self, path: impl Into<String>) -> Self {
        self.preview_path = Some(path.into());
        self
    }
}

/// Template catalog with built-in game templates
#[derive(Debug, Clone, Default)]
pub struct TemplateCatalog {
    /// Available templates
    pub templates: Vec<GameTemplate>,
}

impl TemplateCatalog {
    /// Create catalog with default templates from spec Appendix C
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut catalog = Self::default();

        // Catch Stars (Level 1) - from spec
        catalog.templates.push(
            GameTemplate::new("catch-stars", "Catch the Stars", SchemaLevel::Level1)
                .with_description("Guide your bunny to catch falling stars!")
                .with_yaml(include_str!("../../../templates/catch-stars.yaml")),
        );

        // Avoid Spiders (Level 1) - from spec Appendix C.1
        catalog.templates.push(
            GameTemplate::new("avoid-spiders", "Avoid the Spiders", SchemaLevel::Level1)
                .with_description("Help the bunny avoid scary spiders!")
                .with_yaml(AVOID_SPIDERS_TEMPLATE),
        );

        // Pong (Level 2) - from spec
        catalog.templates.push(
            GameTemplate::new("pong", "Pong Classic", SchemaLevel::Level2)
                .with_description("The classic paddle game for two players!")
                .with_yaml(include_str!("../../../templates/pong.yaml")),
        );

        // Maze (Level 2) - from spec
        catalog.templates.push(
            GameTemplate::new("maze", "Maze Explorer", SchemaLevel::Level2)
                .with_description("Find your way through the maze!")
                .with_yaml(include_str!("../../../templates/maze.yaml")),
        );

        // Platformer (Level 3) - from spec Appendix C.2
        catalog.templates.push(
            GameTemplate::new("platformer", "Jump Adventure", SchemaLevel::Level3)
                .with_description("Jump across platforms and collect coins!")
                .with_yaml(PLATFORMER_TEMPLATE),
        );

        // Dungeon Crawler (Level 3) - from spec Appendix C.3
        catalog.templates.push(
            GameTemplate::new("dungeon", "Dungeon Crawler", SchemaLevel::Level3)
                .with_description("Explore the dungeon and defeat monsters!")
                .with_yaml(DUNGEON_TEMPLATE),
        );

        catalog
    }

    /// Get template by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&GameTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get templates for a specific level
    #[must_use]
    pub fn for_level(&self, level: SchemaLevel) -> Vec<&GameTemplate> {
        self.templates.iter().filter(|t| t.level == level).collect()
    }

    /// Get all template IDs
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.templates.iter().map(|t| t.id.as_str()).collect()
    }
}

// Template YAML constants from spec Appendix C

const AVOID_SPIDERS_TEMPLATE: &str = r#"# Avoid the Spiders - Level 1
# Help the bunny stay safe!

game: avoid-spiders
character: bunny
move: arrows
avoid: spiders

spawn:
  spiders:
    rate: 2 per second
    from: top
    speed: slow

lives: 3

when_avoid:
  sound: oops
  lives: -1
  shake: true

when_game_over:
  sound: sad
  message: "Oh no! The spiders got you!"

background: forest
music: adventure
"#;

const PLATFORMER_TEMPLATE: &str = r#"# Jump Adventure - Level 3
# A classic platformer game

game: platformer
version: "1.0"

characters:
  player:
    type: hero
    move: arrows
    abilities:
      - jump
      - double_jump

world:
  gravity: 20
  type: sidescroll

platforms:
  ground:
    y: 0
    width: full
  floating:
    pattern: random
    count: 10
    width: 100..200

collectibles:
  coins:
    spawn: on_platforms
    value: 10
  powerups:
    types:
      - speed_boost
      - shield
    spawn_rate: 0.1

enemies:
  slimes:
    behavior: patrol
    damage: 1
    on_stomp: defeat

goal:
  type: reach_end
  position: right_edge

lives: 3
score: 0

background: sky
music: upbeat
"#;

const DUNGEON_TEMPLATE: &str = r#"# Dungeon Crawler - Level 3
# Explore and conquer!

game: dungeon-crawler
version: "1.0"

characters:
  hero:
    type: knight
    move: wasd
    stats:
      health: 100
      attack: 10
      defense: 5

world:
  type: dungeon
  generator: procedural
  rooms: 10
  seed: random

enemies:
  goblins:
    health: 30
    attack: 5
    behavior: chase
    loot:
      gold: 10..20

  boss:
    type: dragon
    health: 200
    attack: 25
    room: final
    loot:
      treasure: true

items:
  sword:
    attack_bonus: 5
    rarity: common
  shield:
    defense_bonus: 3
    rarity: common
  potion:
    heal: 50
    rarity: uncommon

ui:
  minimap: true
  health_bar: true
  inventory: true

goal:
  type: defeat_boss

background: dungeon
music: epic
"#;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 10.1
    // ========================================================================

    mod tutorial_stage_tests {
        use super::*;

        #[test]
        fn test_stage_numbers() {
            assert_eq!(TutorialStage::HelloWorld.number(), 1);
            assert_eq!(TutorialStage::AddGoal.number(), 2);
            assert_eq!(TutorialStage::AddFeedback.number(), 3);
            assert_eq!(TutorialStage::MakeChallenging.number(), 4);
        }

        #[test]
        fn test_stage_names() {
            assert_eq!(TutorialStage::HelloWorld.name(), "Hello World");
            assert_eq!(TutorialStage::MakeChallenging.name(), "Make It Challenging");
        }

        #[test]
        fn test_stage_estimated_times() {
            assert_eq!(TutorialStage::HelloWorld.estimated_seconds(), 30);
            assert_eq!(TutorialStage::AddGoal.estimated_seconds(), 60);
            assert_eq!(TutorialStage::AddFeedback.estimated_seconds(), 90);
            assert_eq!(TutorialStage::MakeChallenging.estimated_seconds(), 120);
        }

        #[test]
        fn test_stage_example_yaml() {
            let yaml = TutorialStage::HelloWorld.example_yaml();
            assert!(yaml.contains("character: bunny"));

            let yaml = TutorialStage::MakeChallenging.example_yaml();
            assert!(yaml.contains("character: bunny"));
            assert!(yaml.contains("avoid: spiders"));
            assert!(yaml.contains("lives: 3"));
        }

        #[test]
        fn test_stage_next() {
            assert_eq!(
                TutorialStage::HelloWorld.next(),
                Some(TutorialStage::AddGoal)
            );
            assert_eq!(TutorialStage::MakeChallenging.next(), None);
        }

        #[test]
        fn test_stage_all() {
            let stages = TutorialStage::all();
            assert_eq!(stages.len(), 4);
            assert_eq!(stages[0], TutorialStage::HelloWorld);
            assert_eq!(stages[3], TutorialStage::MakeChallenging);
        }

        #[test]
        fn test_stage_ordering() {
            assert!(TutorialStage::HelloWorld < TutorialStage::AddGoal);
            assert!(TutorialStage::AddGoal < TutorialStage::MakeChallenging);
        }
    }

    mod stage_check_tests {
        use super::*;

        #[test]
        fn test_hello_world_complete() {
            let result = check_yaml_for_stage("character: bunny", TutorialStage::HelloWorld);
            assert!(result.complete);
            assert!(result.missing_elements.is_empty());
        }

        #[test]
        fn test_hello_world_incomplete() {
            let result = check_yaml_for_stage("", TutorialStage::HelloWorld);
            assert!(!result.complete);
            assert!(result.missing_elements.contains(&"character".to_string()));
        }

        #[test]
        fn test_add_goal_complete() {
            let yaml = "character: bunny\ncollect: stars";
            let result = check_yaml_for_stage(yaml, TutorialStage::AddGoal);
            assert!(result.complete);
        }

        #[test]
        fn test_add_goal_missing_collect() {
            let yaml = "character: bunny";
            let result = check_yaml_for_stage(yaml, TutorialStage::AddGoal);
            assert!(!result.complete);
            assert!(result.missing_elements.contains(&"collect".to_string()));
        }

        #[test]
        fn test_add_feedback_complete() {
            let yaml = "character: bunny\ncollect: stars\nwhen_collect:\n  sound: twinkle";
            let result = check_yaml_for_stage(yaml, TutorialStage::AddFeedback);
            assert!(result.complete);
        }

        #[test]
        fn test_make_challenging_complete() {
            let yaml = TutorialStage::MakeChallenging.example_yaml();
            let result = check_yaml_for_stage(yaml, TutorialStage::MakeChallenging);
            assert!(result.complete);
        }

        #[test]
        fn test_make_challenging_incomplete() {
            let yaml = "character: bunny\ncollect: stars";
            let result = check_yaml_for_stage(yaml, TutorialStage::MakeChallenging);
            assert!(!result.complete);
            assert!(result.missing_elements.contains(&"avoid".to_string()));
            assert!(result.missing_elements.contains(&"lives".to_string()));
        }

        #[test]
        fn test_stage_check_message_complete() {
            let result = check_yaml_for_stage("character: bunny", TutorialStage::HelloWorld);
            assert!(result.message.contains("Great job"));
        }

        #[test]
        fn test_stage_check_message_incomplete() {
            let result = check_yaml_for_stage("", TutorialStage::HelloWorld);
            assert!(result.message.contains("Almost there"));
        }
    }

    mod tutorial_progress_tests {
        use super::*;

        #[test]
        fn test_new_progress() {
            let progress = TutorialProgress::new();
            assert_eq!(progress.current_stage, TutorialStage::HelloWorld);
            assert!(progress.completed_stages.is_empty());
        }

        #[test]
        fn test_update_yaml() {
            let mut progress = TutorialProgress::new();
            progress.update_yaml("character: bunny");
            assert_eq!(progress.current_yaml, "character: bunny");
        }

        #[test]
        fn test_check_completion() {
            let mut progress = TutorialProgress::new();
            progress.update_yaml("character: bunny");
            let result = progress.check_stage_completion();
            assert!(result.complete);
        }

        #[test]
        fn test_advance_stage() {
            let mut progress = TutorialProgress::new();
            progress.update_yaml("character: bunny");
            let next = progress.advance().unwrap();
            assert_eq!(next, Some(TutorialStage::AddGoal));
            assert_eq!(progress.current_stage, TutorialStage::AddGoal);
            assert_eq!(progress.completed_stages.len(), 1);
        }

        #[test]
        fn test_advance_fails_when_incomplete() {
            let mut progress = TutorialProgress::new();
            progress.update_yaml("");
            let result = progress.advance();
            assert!(result.is_err());
        }

        #[test]
        fn test_get_hints() {
            let mut progress = TutorialProgress::new();
            let hint1 = progress.get_hint();
            assert!(hint1.is_some());
            let hint2 = progress.get_hint();
            assert!(hint2.is_some());
            assert_ne!(hint1, hint2);
        }

        #[test]
        fn test_hints_exhausted() {
            let mut progress = TutorialProgress::new();
            // Get all hints
            while progress.get_hint().is_some() {}
            // Should return None when exhausted
            assert!(progress.get_hint().is_none());
        }

        #[test]
        fn test_progress_percent() {
            let mut progress = TutorialProgress::new();
            assert_eq!(progress.progress_percent(), 0);

            progress.update_yaml("character: bunny");
            assert_eq!(progress.progress_percent(), 25); // Stage 1 ready

            let _ = progress.advance().unwrap();
            assert_eq!(progress.progress_percent(), 25); // Stage 1 done, stage 2 not ready
        }

        #[test]
        fn test_is_complete() {
            let mut progress = TutorialProgress::new();
            assert!(!progress.is_complete());

            // Complete all stages
            progress.update_yaml(TutorialStage::HelloWorld.example_yaml());
            let _ = progress.advance().unwrap();
            progress.update_yaml(TutorialStage::AddGoal.example_yaml());
            let _ = progress.advance().unwrap();
            progress.update_yaml(TutorialStage::AddFeedback.example_yaml());
            let _ = progress.advance().unwrap();
            progress.update_yaml(TutorialStage::MakeChallenging.example_yaml());
            let _ = progress.advance().unwrap();

            assert!(progress.is_complete());
        }
    }

    mod template_tests {
        use super::*;

        #[test]
        fn test_template_creation() {
            let template = GameTemplate::new("test", "Test Game", SchemaLevel::Level1)
                .with_description("A test game")
                .with_yaml("character: bunny");

            assert_eq!(template.id, "test");
            assert_eq!(template.name, "Test Game");
            assert_eq!(template.description, "A test game");
            assert_eq!(template.level, SchemaLevel::Level1);
        }

        #[test]
        fn test_catalog_with_defaults() {
            let catalog = TemplateCatalog::with_defaults();
            assert!(!catalog.templates.is_empty());
            assert!(catalog.templates.len() >= 4); // At least 4 default templates
        }

        #[test]
        fn test_catalog_get_by_id() {
            let catalog = TemplateCatalog::with_defaults();
            let template = catalog.get("catch-stars");
            assert!(template.is_some());
            assert_eq!(template.unwrap().name, "Catch the Stars");
        }

        #[test]
        fn test_catalog_get_unknown() {
            let catalog = TemplateCatalog::with_defaults();
            let template = catalog.get("nonexistent");
            assert!(template.is_none());
        }

        #[test]
        fn test_catalog_for_level() {
            let catalog = TemplateCatalog::with_defaults();

            let level1 = catalog.for_level(SchemaLevel::Level1);
            assert!(!level1.is_empty());
            assert!(level1.iter().all(|t| t.level == SchemaLevel::Level1));

            let level2 = catalog.for_level(SchemaLevel::Level2);
            assert!(!level2.is_empty());

            let level3 = catalog.for_level(SchemaLevel::Level3);
            assert!(!level3.is_empty());
        }

        #[test]
        fn test_catalog_ids() {
            let catalog = TemplateCatalog::with_defaults();
            let ids = catalog.ids();
            assert!(ids.contains(&"catch-stars"));
            assert!(ids.contains(&"pong"));
        }
    }

    mod template_content_tests {
        use super::*;

        #[test]
        fn test_avoid_spiders_template() {
            assert!(AVOID_SPIDERS_TEMPLATE.contains("game: avoid-spiders"));
            assert!(AVOID_SPIDERS_TEMPLATE.contains("avoid: spiders"));
            assert!(AVOID_SPIDERS_TEMPLATE.contains("lives: 3"));
        }

        #[test]
        fn test_platformer_template() {
            assert!(PLATFORMER_TEMPLATE.contains("game: platformer"));
            assert!(PLATFORMER_TEMPLATE.contains("abilities:"));
            assert!(PLATFORMER_TEMPLATE.contains("double_jump"));
        }

        #[test]
        fn test_dungeon_template() {
            assert!(DUNGEON_TEMPLATE.contains("game: dungeon-crawler"));
            assert!(DUNGEON_TEMPLATE.contains("generator: procedural"));
            assert!(DUNGEON_TEMPLATE.contains("boss:"));
        }
    }

    mod tutorial_error_tests {
        use super::*;

        #[test]
        fn test_stage_not_complete_display() {
            let err = TutorialError::StageNotComplete {
                stage: TutorialStage::HelloWorld,
                missing: vec!["character".to_string()],
            };
            let msg = err.to_string();
            assert!(msg.contains("Hello World"));
            assert!(msg.contains("character"));
        }

        #[test]
        fn test_invalid_yaml_display() {
            let err = TutorialError::InvalidYaml {
                message: "bad syntax".to_string(),
            };
            let msg = err.to_string();
            assert!(msg.contains("bad syntax"));
        }

        #[test]
        fn test_tutorial_error_to_yaml_error() {
            let err = TutorialError::StageNotComplete {
                stage: TutorialStage::HelloWorld,
                missing: vec!["character".to_string()],
            };
            let yaml_err: YamlError = err.into();
            assert!(matches!(yaml_err, YamlError::ValidationError { .. }));
        }
    }
}
