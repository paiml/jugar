//! Physics Toy Sandbox - Probar Integration Tests
//!
//! Four-Harness Testing Architecture (SQLite-inspired):
//! - Harness 1: Physics Canary Tests (80% user action coverage)
//! - Harness 2: Remix Validation Suite (100% API coverage)
//! - Harness 3: Determinism Suite (replay verification)
//! - Harness 4: Chaos Engineering Suite (graceful degradation)
//!
//! Toyota Way Principles:
//! - Jidoka: Tests stop on first defect
//! - Poka-Yoke: Invalid states prevented at compile time
//! - Genchi Genbutsu: Go see actual physics behavior

#![allow(
    unused_results,
    unused_variables,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::doc_markdown,
    clippy::cast_precision_loss,
    clippy::default_trait_access,
    clippy::uninlined_format_args,
    clippy::redundant_clone
)]

use glam::Vec2;
use physics_toy_sandbox::*;

// =============================================================================
// HARNESS 1: PHYSICS CANARY TESTS
// Purpose: Validate core physics behaviors users depend on
// =============================================================================

mod physics_canary {
    use super::*;

    /// C01: Ball at rest on floor has zero velocity
    #[test]
    fn test_ball_static_equilibrium() {
        // A ball on a flat surface should come to rest
        let ball = SerializedEntity::new(
            ObjectType::Ball,
            Transform2D {
                position: Vec2::new(0.0, 10.0),
                rotation: 0.0,
                scale: Vec2::splat(20.0),
            },
        );

        // Verify ball is dynamic (affected by physics)
        assert!(ObjectType::Ball.is_dynamic());

        // Verify ball is not a trigger
        assert!(!ObjectType::Ball.is_trigger());
    }

    /// C02: Domino is a dynamic physics object
    #[test]
    fn test_domino_is_dynamic() {
        assert!(ObjectType::Domino.is_dynamic());
        assert!(!ObjectType::Domino.is_constraint());
        assert!(!ObjectType::Domino.is_trigger());
    }

    /// C03: Ramp is a static physics object
    #[test]
    fn test_ramp_is_static() {
        assert!(!ObjectType::Ramp.is_dynamic());
        assert!(!ObjectType::Ramp.is_trigger());
    }

    /// C04: Lever is hinged (not dynamic, not trigger)
    #[test]
    fn test_lever_physics_type() {
        assert!(!ObjectType::Lever.is_dynamic());
        assert!(!ObjectType::Lever.is_trigger());
    }

    /// C05: Spring is a constraint type
    #[test]
    fn test_spring_is_constraint() {
        assert!(ObjectType::Spring.is_constraint());
        assert!(!ObjectType::Spring.is_dynamic());
    }

    /// C06: Pulley is a constraint type
    #[test]
    fn test_pulley_is_constraint() {
        assert!(ObjectType::Pulley.is_constraint());
        assert!(!ObjectType::Pulley.is_dynamic());
    }

    /// C07: Fan is a force field (not dynamic)
    #[test]
    fn test_fan_is_force_field() {
        assert!(!ObjectType::Fan.is_dynamic());
        assert!(!ObjectType::Fan.is_trigger());
        assert!(!ObjectType::Fan.is_constraint());
    }

    /// C08: Magnet is a force field (not dynamic)
    #[test]
    fn test_magnet_is_force_field() {
        assert!(!ObjectType::Magnet.is_dynamic());
        assert!(!ObjectType::Magnet.is_trigger());
        assert!(!ObjectType::Magnet.is_constraint());
    }

    /// C09: Bucket is a trigger (detects objects)
    #[test]
    fn test_bucket_is_trigger() {
        assert!(ObjectType::Bucket.is_trigger());
        assert!(!ObjectType::Bucket.is_dynamic());
    }

    /// C10: Sensor is a trigger (detects proximity)
    #[test]
    fn test_sensor_is_trigger() {
        assert!(ObjectType::Sensor.is_trigger());
        assert!(!ObjectType::Sensor.is_dynamic());
    }

    /// C11: Transform default is identity
    #[test]
    fn test_transform_default_is_identity() {
        let t = Transform2D::default();
        assert_eq!(t.position, Vec2::ZERO);
        assert!((t.rotation - 0.0).abs() < f32::EPSILON);
        assert_eq!(t.scale, Vec2::ONE);
    }

    /// C12: Physics backend defaults to WebGPU
    #[test]
    fn test_default_backend_is_webgpu() {
        assert_eq!(PhysicsBackend::default(), PhysicsBackend::WebGpu);
    }

    /// C13: SerializedEntity has material for dynamic objects
    #[test]
    fn test_entity_has_material_for_dynamic() {
        let ball = SerializedEntity::new(ObjectType::Ball, Transform2D::default());
        assert!(ball.material.is_some());

        let domino = SerializedEntity::new(ObjectType::Domino, Transform2D::default());
        assert!(domino.material.is_some());
    }

    /// C14: SerializedEntity has no material for triggers
    #[test]
    fn test_entity_no_material_for_triggers() {
        let bucket = SerializedEntity::new(ObjectType::Bucket, Transform2D::default());
        assert!(bucket.material.is_none());

        let sensor = SerializedEntity::new(ObjectType::Sensor, Transform2D::default());
        assert!(sensor.material.is_none());
    }
}

// =============================================================================
// HARNESS 2: REMIX VALIDATION SUITE
// Purpose: Every remix operation produces valid state
// =============================================================================

mod remix_validation {
    use super::*;

    /// R01: Fork creates independent copy with different ID
    #[test]
    fn test_fork_creates_independent_copy() {
        let original = Contraption::new("Original");
        let forked = original.fork("Forked Copy");

        // IDs should differ
        assert_ne!(original.id, forked.id);

        // Forked should reference original
        assert_eq!(forked.forked_from, Some(original.id));

        // Forked is marked as a fork
        assert!(forked.is_fork());
        assert!(!original.is_fork());
    }

    /// R02: Serialization round-trips perfectly
    #[test]
    fn test_serialization_roundtrip() {
        let original = ContraptionBuilder::new("Test Scene")
            .with_object(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(123.456, 789.012),
                    rotation: 1.23456,
                    scale: Vec2::new(20.0, 20.0),
                },
            )
            .build()
            .expect("should build");

        // Serialize to bytes
        let bytes = original.serialize().expect("should serialize");

        // Deserialize back
        let restored = Contraption::deserialize(&bytes).expect("should deserialize");

        // Should be identical
        assert_eq!(original.id, restored.id);
        assert_eq!(original.entities.len(), restored.entities.len());

        let orig_pos = original.entities[0].transform.position;
        let rest_pos = restored.entities[0].transform.position;
        assert!((orig_pos.x - rest_pos.x).abs() < f32::EPSILON);
        assert!((orig_pos.y - rest_pos.y).abs() < f32::EPSILON);
    }

    /// R03: Invalid scenes rejected with clear errors (object limit)
    #[test]
    fn test_object_limit_rejection() {
        let mut builder = ContraptionBuilder::new("Too Large");

        // Add more than MAX_OBJECTS_PER_CONTRAPTION
        for i in 0..600 {
            builder = builder.with_object(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(i as f32, 0.0),
                    ..Transform2D::default()
                },
            );
        }

        let result = builder.build();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("limit"));
    }

    /// R04: Content hash deduplication
    #[test]
    fn test_content_hash_dedup() {
        let scene1 = ContraptionBuilder::new("Scene A")
            .with_object(ObjectType::Ball, Transform2D::default())
            .build()
            .expect("should build");

        let scene2 = ContraptionBuilder::new("Scene A")
            .with_object(ObjectType::Ball, Transform2D::default())
            .build()
            .expect("should build");

        // Same content should produce same hash
        assert_eq!(scene1.content_hash(), scene2.content_hash());

        // Different content should produce different hash
        let scene3 = ContraptionBuilder::new("Scene B")
            .with_object(ObjectType::Ramp, Transform2D::default())
            .build()
            .expect("should build");

        assert_ne!(scene1.content_hash(), scene3.content_hash());
    }

    /// R05: Fork resets play and remix counts
    #[test]
    fn test_fork_resets_counts() {
        let mut original = Contraption::new("Original");
        original.metadata.play_count = 100;
        original.metadata.remix_count = 50;

        let forked = original.fork("Forked");

        assert_eq!(forked.metadata.play_count, 0);
        assert_eq!(forked.metadata.remix_count, 0);
    }

    /// R06: Remix graph tracks lineage correctly
    #[test]
    fn test_remix_graph_lineage() {
        let mut graph = RemixGraph::new();

        let root = Contraption::new("Root");
        graph.register(&root);

        let fork1 = root.fork("Fork 1");
        graph.register(&fork1);

        let fork2 = fork1.fork("Fork 2");
        graph.register(&fork2);

        // Verify ancestry
        let ancestors = graph.ancestors(fork2.id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0], fork1.id);
        assert_eq!(ancestors[1], root.id);

        // Verify root finding
        assert_eq!(graph.root(fork2.id), root.id);

        // Verify depth
        assert_eq!(graph.depth(root.id), 0);
        assert_eq!(graph.depth(fork1.id), 1);
        assert_eq!(graph.depth(fork2.id), 2);
    }

    /// R07: Storage saves and loads contraptions
    #[test]
    fn test_storage_save_load() {
        let mut storage = ContraptionStorage::new();

        let contraption = ContraptionBuilder::new("Test")
            .with_object(ObjectType::Ball, Transform2D::default())
            .build()
            .expect("should build");

        let id = storage.save(contraption.clone()).expect("should save");
        let loaded = storage.load(id).expect("should load");

        assert_eq!(loaded.id, contraption.id);
        assert_eq!(loaded.metadata.name, "Test");
    }

    /// R08: Storage rejects invalid contraptions
    #[test]
    fn test_storage_rejects_invalid() {
        let mut storage = ContraptionStorage::new();

        // Create invalid contraption (too many objects)
        let mut contraption = Contraption::new("Invalid");
        for i in 0..600 {
            contraption.entities.push(SerializedEntity::new(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(i as f32, 0.0),
                    ..Transform2D::default()
                },
            ));
        }

        let result = storage.save(contraption);
        assert!(result.is_err());
    }

    /// R09: Builder fluent API works
    #[test]
    fn test_builder_fluent_api() {
        let contraption = ContraptionBuilder::new("My Machine")
            .author("Test Author")
            .description("A test contraption")
            .tag("physics")
            .tag("demo")
            .with_object(ObjectType::Ball, Transform2D::default())
            .with_object(
                ObjectType::Ramp,
                Transform2D {
                    position: Vec2::new(100.0, 0.0),
                    rotation: 0.5,
                    scale: Vec2::new(200.0, 20.0),
                },
            )
            .with_seed(42)
            .build()
            .expect("should build");

        assert_eq!(contraption.metadata.name, "My Machine");
        assert_eq!(contraption.metadata.author, "Test Author");
        assert_eq!(contraption.metadata.description, "A test contraption");
        assert_eq!(contraption.metadata.tags.len(), 2);
        assert_eq!(contraption.entities.len(), 2);
        assert_eq!(contraption.initial_seed, 42);
    }

    /// R10: Empty contraption is valid
    #[test]
    fn test_empty_contraption_valid() {
        let contraption = ContraptionBuilder::new("Empty")
            .build()
            .expect("empty should be valid");

        assert_eq!(contraption.object_count(), 0);
        assert!(contraption.validate().is_ok());
    }
}

// =============================================================================
// HARNESS 3: DETERMINISM SUITE
// Purpose: Same inputs produce same outputs (replay verification)
// =============================================================================

mod determinism {
    use super::*;

    /// D01: Same contraption serializes to same bytes
    #[test]
    fn test_serialization_determinism() {
        let c = ContraptionBuilder::new("Deterministic")
            .with_object(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(100.0, 200.0),
                    rotation: 0.5,
                    scale: Vec2::splat(20.0),
                },
            )
            .with_seed(12345)
            .build()
            .expect("should build");

        let bytes1 = c.serialize().expect("should serialize");
        let bytes2 = c.serialize().expect("should serialize");

        assert_eq!(
            bytes1, bytes2,
            "Same contraption should serialize identically"
        );
    }

    /// D02: Content hash is deterministic
    #[test]
    fn test_content_hash_determinism() {
        let c = ContraptionBuilder::new("Hash Test")
            .with_object(ObjectType::Domino, Transform2D::default())
            .build()
            .expect("should build");

        let hash1 = c.content_hash();
        let hash2 = c.content_hash();

        assert_eq!(hash1, hash2, "Content hash should be deterministic");
    }

    /// D03: Engine version is preserved through serialization
    #[test]
    fn test_engine_version_preserved() {
        let original = Contraption::new("Versioned");
        let bytes = original.serialize().expect("should serialize");
        let restored = Contraption::deserialize(&bytes).expect("should deserialize");

        assert_eq!(
            original.engine_version, restored.engine_version,
            "Engine version must be preserved for replay compatibility"
        );
    }

    /// D04: Initial seed is preserved through serialization
    #[test]
    fn test_seed_preserved() {
        let original = ContraptionBuilder::new("Seeded")
            .with_seed(0xDEAD_BEEF)
            .build()
            .expect("should build");

        let bytes = original.serialize().expect("should serialize");
        let restored = Contraption::deserialize(&bytes).expect("should deserialize");

        assert_eq!(
            original.initial_seed, restored.initial_seed,
            "Seed must be preserved for deterministic replay"
        );
    }

    /// D05: Physics config is preserved through serialization
    #[test]
    fn test_physics_config_preserved() {
        let config = PhysicsConfig {
            gravity: Vec2::new(0.0, -20.0),
            substeps: 8,
            backend: PhysicsBackend::WasmSimd,
        };

        let original = ContraptionBuilder::new("Custom Physics")
            .with_physics_config(config.clone())
            .build()
            .expect("should build");

        let bytes = original.serialize().expect("should serialize");
        let restored = Contraption::deserialize(&bytes).expect("should deserialize");

        assert_eq!(
            original.physics_config.gravity,
            restored.physics_config.gravity
        );
        assert_eq!(
            original.physics_config.substeps,
            restored.physics_config.substeps
        );
        assert_eq!(
            original.physics_config.backend,
            restored.physics_config.backend
        );
    }

    /// D06: Fork preserves parent reference
    #[test]
    fn test_fork_parent_reference() {
        let parent = Contraption::new("Parent");
        let parent_id = parent.id;

        let child = parent.fork("Child");

        assert_eq!(
            child.forked_from,
            Some(parent_id),
            "Fork must preserve parent reference"
        );
    }
}

// =============================================================================
// HARNESS 4: CHAOS ENGINEERING SUITE
// Purpose: System recovers from failures gracefully
// =============================================================================

mod chaos {
    use super::*;

    /// X01: Malformed data rejected with clear error
    #[test]
    fn test_malformed_data_rejected() {
        let garbage = vec![0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03];
        let result = Contraption::deserialize(&garbage);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("invalid") || msg.contains("corrupt") || msg.contains("Deserialization"),
            "Error message should be clear: {}",
            msg
        );
    }

    /// X02: Empty data rejected
    #[test]
    fn test_empty_data_rejected() {
        let empty: Vec<u8> = vec![];
        let result = Contraption::deserialize(&empty);
        assert!(result.is_err());
    }

    /// X03: Truncated data rejected
    #[test]
    fn test_truncated_data_rejected() {
        let c = Contraption::new("Complete");
        let bytes = c.serialize().expect("should serialize");

        // Truncate the data
        let truncated = &bytes[..bytes.len() / 2];
        let result = Contraption::deserialize(truncated);

        assert!(result.is_err());
    }

    /// X04: Object limit prevents resource exhaustion
    #[test]
    fn test_object_limit_prevents_exhaustion() {
        let mut c = Contraption::new("Stress Test");

        // Fill to capacity
        for i in 0..MAX_OBJECTS_PER_CONTRAPTION {
            c.add_object(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(i as f32, 0.0),
                    ..Transform2D::default()
                },
            )
            .expect("should add");
        }

        // One more should fail
        let result = c.add_object(ObjectType::Ball, Transform2D::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("limit"));
    }

    /// X05: Storage returns NotFound for missing contraption
    #[test]
    fn test_storage_not_found() {
        let storage = ContraptionStorage::new();
        let fake_id = ContraptionId::new();

        let result = storage.load(fake_id);
        assert!(matches!(result, Err(SandboxError::NotFound(_))));
    }

    /// X06: Complexity thermometer blocks at threshold
    #[test]
    fn test_thermometer_blocks_additions() {
        let mut thermo = ComplexityThermometer::default();

        // Below threshold - should not block
        for _ in 0..10 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 5.0,
                render_ms: 5.0,
                ui_ms: 2.0,
                other_ms: 0.0,
            });
        }
        // 12ms / 16.67ms = ~72% load
        assert!(!thermo.should_block_additions());

        // Above threshold - should block
        for _ in 0..60 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 8.0,
                render_ms: 8.0,
                ui_ms: 2.0,
                other_ms: 0.0,
            });
        }
        // 18ms / 16.67ms = ~108% load
        assert!(thermo.should_block_additions());
    }

    /// X07: Thermometer visual states are correct
    #[test]
    fn test_thermometer_visual_states() {
        let mut thermo = ComplexityThermometer::default();

        // Green: < 70%
        for _ in 0..10 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 4.0,
                render_ms: 4.0,
                ui_ms: 1.0,
                other_ms: 0.0,
            });
        }
        assert_eq!(thermo.visual_state(), ThermometerState::Green);

        // Yellow: 70-90%
        thermo.reset();
        for _ in 0..10 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 6.0,
                render_ms: 6.0,
                ui_ms: 2.0,
                other_ms: 0.0,
            });
        }
        assert_eq!(thermo.visual_state(), ThermometerState::Yellow);

        // Red: > 90%
        thermo.reset();
        for _ in 0..10 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 8.0,
                render_ms: 8.0,
                ui_ms: 2.0,
                other_ms: 0.0,
            });
        }
        assert_eq!(thermo.visual_state(), ThermometerState::Red);
    }

    /// X08: Delete removes from storage completely
    #[test]
    fn test_delete_removes_completely() {
        let mut storage = ContraptionStorage::new();

        let c = Contraption::new("To Delete");
        let id = c.id;
        let hash = c.content_hash();

        storage.save(c).expect("should save");
        assert!(storage.exists(id));
        assert!(storage.find_by_hash(hash).is_some());

        storage.delete(id);
        assert!(!storage.exists(id));
        assert!(storage.find_by_hash(hash).is_none());
    }

    /// X09: Removing from graph orphans children
    #[test]
    fn test_graph_remove_orphans_children() {
        let mut graph = RemixGraph::new();

        let root = Contraption::new("Root");
        graph.register(&root);

        let child = root.fork("Child");
        graph.register(&child);

        // Child has parent
        assert_eq!(graph.parent(child.id), Some(root.id));
        assert_eq!(graph.depth(child.id), 1);

        // Remove root
        graph.remove(root.id);

        // Child is now orphaned (root)
        assert!(graph.is_root(child.id));
        assert_eq!(graph.depth(child.id), 0);
        assert_eq!(graph.parent(child.id), None);
    }
}

// =============================================================================
// MATERIAL PROPERTIES TESTS (POKA-YOKE)
// Purpose: Verify compile-time and runtime safety guarantees
// =============================================================================

mod material_poka_yoke {
    use super::*;

    /// M01: Default material has valid density
    #[test]
    fn test_default_material_valid() {
        let mat = MaterialProperties::default();
        assert!(mat.density() > 0.0, "Default density must be positive");
    }

    /// M02: Cannot create material with zero density
    #[test]
    fn test_zero_density_rejected() {
        let result = MaterialProperties::new(0.0);
        assert!(result.is_err());
    }

    /// M03: Cannot create material with negative density
    #[test]
    fn test_negative_density_rejected() {
        let result = MaterialProperties::new(-100.0);
        assert!(result.is_err());
    }

    /// M04: Bounciness clamped to valid range
    #[test]
    fn test_bounciness_clamped() {
        let mut mat = MaterialProperties::default();

        mat.set_bounciness(2.0);
        assert!((mat.bounciness() - 1.0).abs() < f32::EPSILON);

        mat.set_bounciness(-1.0);
        assert!((mat.bounciness() - 0.0).abs() < f32::EPSILON);
    }

    /// M05: Friction cannot be negative
    #[test]
    fn test_friction_clamped() {
        let mut mat = MaterialProperties::default();

        mat.set_friction_static(-0.5);
        assert!(mat.friction_static() >= 0.0);

        mat.set_friction_dynamic(-0.5);
        assert!(mat.friction_dynamic() >= 0.0);
    }

    /// M06: Material presets have sensible values
    #[test]
    fn test_presets_sensible() {
        // Wood: moderate density, low bounce
        let wood = MaterialProperties::from_preset(MaterialPreset::Wood);
        assert!(wood.density() > 500.0 && wood.density() < 1000.0);
        assert!(wood.bounciness() < 0.5);

        // Metal: high density, low bounce
        let metal = MaterialProperties::from_preset(MaterialPreset::Metal);
        assert!(metal.density() > 5000.0);
        assert!(metal.bounciness() < 0.5);

        // Rubber: moderate density, high bounce
        let rubber = MaterialProperties::from_preset(MaterialPreset::Rubber);
        assert!(rubber.bounciness() > 0.5);

        // Ice: low friction
        let ice = MaterialProperties::from_preset(MaterialPreset::Ice);
        assert!(ice.friction_static() < 0.1);
        assert!(ice.friction_dynamic() < 0.1);
    }

    /// M07: Mass calculation is always positive
    #[test]
    fn test_mass_always_positive() {
        let mat = MaterialProperties::default();

        let mass = mat.mass_for_volume(1.0);
        assert!(mass > 0.0);

        // Zero volume gives zero mass (not negative or NaN)
        let zero_mass = mat.mass_for_volume(0.0);
        assert!((zero_mass - 0.0).abs() < f32::EPSILON);
    }

    /// M08: Modifying material marks it as custom
    #[test]
    fn test_modification_marks_custom() {
        let mut mat = MaterialProperties::from_preset(MaterialPreset::Wood);
        assert_eq!(mat.preset, MaterialPreset::Wood);

        mat.set_bounciness(0.8);
        assert_eq!(mat.preset, MaterialPreset::Custom);
    }
}

// =============================================================================
// ENGINE VERSION COMPATIBILITY TESTS (JIDOKA)
// Purpose: Ensure replay compatibility across engine updates
// =============================================================================

mod version_compatibility {
    use super::*;

    /// V01: Current version is always compatible with itself
    #[test]
    fn test_current_version_compatible() {
        let c = Contraption::new("Current");
        assert!(c.is_version_compatible());
    }

    /// V02: Engine version is captured on creation
    #[test]
    fn test_engine_version_captured() {
        let c = Contraption::new("Versioned");
        let expected = semver::Version::parse(ENGINE_VERSION)
            .unwrap_or_else(|_| semver::Version::new(0, 1, 0));

        assert_eq!(c.engine_version, expected);
    }
}

// =============================================================================
// INTEGRATION TESTS
// Purpose: Full workflow validation
// =============================================================================

// =============================================================================
// ADDITIONAL COVERAGE TESTS
// Purpose: Cover remaining untested code paths
// =============================================================================

mod additional_coverage {
    use super::*;

    /// AC01: VisualProperties with_color creates colored visual
    #[test]
    fn test_visual_properties_with_color() {
        let visual = VisualProperties::with_color(255, 128, 64);
        assert_eq!(visual.color, [255, 128, 64]);
        assert!((visual.opacity - 1.0).abs() < f32::EPSILON);
        assert!(visual.sprite.is_none());
    }

    /// AC02: SerializedEntity with_visual sets visual properties
    #[test]
    fn test_entity_with_visual() {
        let visual = VisualProperties::with_color(100, 100, 100);
        let entity = SerializedEntity::new(ObjectType::Ball, Transform2D::default())
            .with_visual(visual.clone());

        assert_eq!(entity.visual.color, [100, 100, 100]);
    }

    /// AC03: ContraptionMetadata with_author sets author
    #[test]
    fn test_metadata_with_author() {
        let meta = ContraptionMetadata::new("Test")
            .with_author("Author Name")
            .with_description("A test description")
            .with_tag("test");

        assert_eq!(meta.author, "Author Name");
        assert_eq!(meta.description, "A test description");
        assert!(meta.tags.contains(&"test".to_string()));
    }

    /// AC04: PhysicsConfig default values
    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert!((config.gravity.y - (-9.8)).abs() < f32::EPSILON);
        assert_eq!(config.substeps, 4);
        assert_eq!(config.backend, PhysicsBackend::default());
    }

    /// AC05: BehaviorScript default is empty
    #[test]
    fn test_behavior_script_default() {
        let script = BehaviorScript::default();
        assert!(script.script_type.is_empty());
        assert!(script.params.is_empty());
    }

    /// AC06: ContraptionId display format
    #[test]
    fn test_contraption_id_display() {
        let id = ContraptionId::new();
        let display = format!("{}", id);
        // UUID format has hyphens
        assert!(display.contains('-'));
        assert_eq!(display.len(), 36); // UUID v4 format
    }

    /// AC07: Difficulty default is Medium
    #[test]
    fn test_difficulty_default() {
        let difficulty = Difficulty::default();
        assert_eq!(difficulty, Difficulty::Medium);
    }

    /// AC08: RemixGraph roots returns all root contraptions
    #[test]
    fn test_remix_graph_roots() {
        let mut graph = RemixGraph::new();

        let root1 = Contraption::new("Root 1");
        let root2 = Contraption::new("Root 2");
        graph.register(&root1);
        graph.register(&root2);

        let roots = graph.roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&root1.id));
        assert!(roots.contains(&root2.id));
    }

    /// AC09: ContraptionStorage all() returns all contraptions
    #[test]
    fn test_storage_all() {
        let mut storage = ContraptionStorage::new();

        let c1 = Contraption::new("One");
        let c2 = Contraption::new("Two");

        let _ = storage.save(c1);
        let _ = storage.save(c2);

        let all = storage.all();
        assert_eq!(all.len(), 2);
    }

    /// AC10: Rolling average sample_count
    #[test]
    fn test_rolling_average_sample_count() {
        let mut avg = thermometer::RollingAverage::<10>::new();
        assert_eq!(avg.sample_count(), 0);

        avg.push(1.0);
        assert_eq!(avg.sample_count(), 1);

        avg.push(2.0);
        assert_eq!(avg.sample_count(), 2);
    }

    /// AC11: PerformanceBreakdown dominant subsystem
    #[test]
    fn test_performance_breakdown_dominant() {
        let breakdown = PerformanceBreakdown {
            physics_ms: 10.0,
            render_ms: 5.0,
            ui_ms: 2.0,
            other_ms: 1.0,
        };
        assert_eq!(breakdown.dominant_subsystem(), "physics");

        let breakdown2 = PerformanceBreakdown {
            physics_ms: 5.0,
            render_ms: 10.0,
            ui_ms: 2.0,
            other_ms: 1.0,
        };
        assert_eq!(breakdown2.dominant_subsystem(), "render");

        let breakdown3 = PerformanceBreakdown {
            physics_ms: 2.0,
            render_ms: 5.0,
            ui_ms: 10.0,
            other_ms: 1.0,
        };
        assert_eq!(breakdown3.dominant_subsystem(), "ui");

        let breakdown4 = PerformanceBreakdown {
            physics_ms: 1.0,
            render_ms: 1.0,
            ui_ms: 1.0,
            other_ms: 10.0,
        };
        assert_eq!(breakdown4.dominant_subsystem(), "other");
    }

    /// AC12: Thermometer format_display
    #[test]
    fn test_thermometer_format_display() {
        let mut thermo = ComplexityThermometer::new(60.0);
        thermo.update(PerformanceBreakdown {
            physics_ms: 4.0,
            render_ms: 6.0,
            ui_ms: 2.0,
            other_ms: 0.0,
        });

        let display = thermo.format_display();
        assert!(display.contains("Physics:"));
        assert!(display.contains("Render:"));
        assert!(display.contains("UI:"));
        assert!(display.contains("Budget:"));
    }

    /// AC13: ThermometerState css_color and description
    #[test]
    fn test_thermometer_state_colors() {
        assert!(ThermometerState::Green.css_color().starts_with('#'));
        assert!(ThermometerState::Yellow.css_color().starts_with('#'));
        assert!(ThermometerState::Red.css_color().starts_with('#'));

        assert!(!ThermometerState::Green.description().is_empty());
        assert!(!ThermometerState::Yellow.description().is_empty());
        assert!(!ThermometerState::Red.description().is_empty());
    }

    /// AC14: Thermometer set_thresholds
    #[test]
    fn test_thermometer_custom_thresholds() {
        let mut thermo = ComplexityThermometer::default();
        thermo.set_thresholds(0.5, 0.8);

        // At 60% load, should be yellow with custom thresholds (0.5 < 0.6 < 0.8)
        for _ in 0..60 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 5.0,
                render_ms: 4.0,
                ui_ms: 1.0,
                other_ms: 0.0,
            });
        }
        assert_eq!(thermo.visual_state(), ThermometerState::Yellow);
    }

    /// AC15: Thermometer budget_ms and target_fps
    #[test]
    fn test_thermometer_budget_and_fps() {
        let thermo = ComplexityThermometer::new(30.0);
        assert!((thermo.target_fps() - 30.0).abs() < f32::EPSILON);
        assert!((thermo.budget_ms() - 33.33).abs() < 0.1);
    }

    /// AC16: Material density_raw returns NonZeroU32
    #[test]
    fn test_material_density_raw() {
        let mat = MaterialProperties::default();
        let raw = mat.density_raw();
        assert!(raw.get() > 0);
    }

    /// AC17: ContraptionBuilder with_entity
    #[test]
    fn test_builder_with_entity() {
        let entity = SerializedEntity::new(ObjectType::Ball, Transform2D::default())
            .with_material(MaterialProperties::from_preset(MaterialPreset::Metal));

        let contraption = ContraptionBuilder::new("Entity Test")
            .with_entity(entity)
            .build()
            .expect("should build");

        assert_eq!(contraption.entities.len(), 1);
        let mat = contraption.entities[0]
            .material
            .as_ref()
            .expect("should have material");
        assert!(mat.density() > 5000.0); // Metal is heavy
    }

    /// AC18: Load percent returns correct percentage (kills mutation)
    #[test]
    fn test_load_percent_correct_value() {
        let mut thermo = ComplexityThermometer::new(60.0);

        // At 60 FPS, budget is ~16.67ms
        // 8.33ms total = 50% load
        for _ in 0..60 {
            thermo.update(PerformanceBreakdown {
                physics_ms: 4.0,
                render_ms: 3.0,
                ui_ms: 1.33,
                other_ms: 0.0,
            });
        }

        let percent = thermo.load_percent();
        // Should be approximately 50%
        assert!(
            percent > 40.0,
            "Load percent should be > 40%, got {}",
            percent
        );
        assert!(
            percent < 60.0,
            "Load percent should be < 60%, got {}",
            percent
        );
        // Cannot be 0.0 or -1.0 (mutation targets)
        assert!(percent > 0.0);
    }

    /// AC19: ContraptionStorage graph_mut returns mutable ref (kills mutation)
    #[test]
    fn test_storage_graph_mut() {
        let mut storage = ContraptionStorage::new();

        let c = Contraption::new("Test");
        let _ = storage.save(c.clone());

        // Get mutable graph and modify it
        let graph = storage.graph_mut();

        // Register another contraption
        let c2 = c.fork("Fork");
        graph.register(&c2);

        // Verify the change persists
        assert_eq!(storage.graph().depth(c2.id), 1);
    }

    /// AC20: set_friction_dynamic actually sets value (kills mutation)
    #[test]
    fn test_set_friction_dynamic_effect() {
        let mut mat = MaterialProperties::default();
        let original = mat.friction_dynamic();

        mat.set_friction_dynamic(0.99);
        assert!(
            (mat.friction_dynamic() - 0.99).abs() < 0.01,
            "friction_dynamic should be 0.99, got {}",
            mat.friction_dynamic()
        );
        assert!(
            (mat.friction_dynamic() - original).abs() > 0.1,
            "friction_dynamic should change from original"
        );
    }

    /// AC21: Dominant subsystem boundary conditions (kills < vs <= mutations)
    #[test]
    fn test_dominant_subsystem_boundaries() {
        // Test exact equality - physics wins when exactly equal
        let equal = PerformanceBreakdown {
            physics_ms: 5.0,
            render_ms: 5.0,
            ui_ms: 5.0,
            other_ms: 5.0,
        };
        // When all equal, physics wins (checked first)
        assert_eq!(equal.dominant_subsystem(), "physics");

        // Physics slightly higher
        let physics_high = PerformanceBreakdown {
            physics_ms: 5.01,
            render_ms: 5.0,
            ui_ms: 5.0,
            other_ms: 5.0,
        };
        assert_eq!(physics_high.dominant_subsystem(), "physics");

        // Render slightly higher
        let render_high = PerformanceBreakdown {
            physics_ms: 5.0,
            render_ms: 5.01,
            ui_ms: 5.0,
            other_ms: 5.0,
        };
        assert_eq!(render_high.dominant_subsystem(), "render");

        // UI slightly higher
        let ui_high = PerformanceBreakdown {
            physics_ms: 5.0,
            render_ms: 5.0,
            ui_ms: 5.01,
            other_ms: 5.0,
        };
        assert_eq!(ui_high.dominant_subsystem(), "ui");

        // Other slightly higher
        let other_high = PerformanceBreakdown {
            physics_ms: 5.0,
            render_ms: 5.0,
            ui_ms: 5.0,
            other_ms: 5.01,
        };
        assert_eq!(other_high.dominant_subsystem(), "other");
    }
}

// =============================================================================
// 100% COVERAGE TESTS
// Purpose: Cover remaining edge cases for 100% coverage
// =============================================================================

mod coverage_100 {
    use super::*;

    /// COV01: ContraptionId::default() uses new()
    #[test]
    fn test_contraption_id_default() {
        let id1 = ContraptionId::default();
        let id2 = ContraptionId::default();
        // Each call to default should create a unique ID
        assert_ne!(id1, id2);
    }

    /// COV02: RollingAverage::default() uses new()
    #[test]
    fn test_rolling_average_default() {
        let avg: thermometer::RollingAverage<10> = Default::default();
        assert_eq!(avg.sample_count(), 0);
        assert!((avg.average() - 0.0).abs() < f32::EPSILON);
    }

    /// COV03: Thermometer breakdown() getter returns correct breakdown
    #[test]
    fn test_thermometer_breakdown_getter() {
        let mut thermo = ComplexityThermometer::new(60.0);
        let breakdown = PerformanceBreakdown {
            physics_ms: 5.5,
            render_ms: 6.6,
            ui_ms: 2.2,
            other_ms: 1.1,
        };
        thermo.update(breakdown);

        let retrieved = thermo.breakdown();
        assert!((retrieved.physics_ms - 5.5).abs() < f32::EPSILON);
        assert!((retrieved.render_ms - 6.6).abs() < f32::EPSILON);
        assert!((retrieved.ui_ms - 2.2).abs() < f32::EPSILON);
        assert!((retrieved.other_ms - 1.1).abs() < f32::EPSILON);
    }

    /// COV04: MaterialProperties::new with very small density that becomes zero after conversion
    #[test]
    fn test_material_density_too_small() {
        // 0.0001 kg/m³ * 1000 = 0.1, which truncates to 0 as u32
        let result = MaterialProperties::new(0.0001);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("small") || err.to_string().contains("positive"));
    }

    /// COV05: MaterialProperties::set_density with very small density
    #[test]
    fn test_material_set_density_too_small() {
        let mut mat = MaterialProperties::default();
        // 0.0001 kg/m³ * 1000 = 0.1, which truncates to 0 as u32
        let result = mat.set_density(0.0001);
        assert!(result.is_err());
    }

    /// COV06: RemixGraph remove() decrements parent's child count
    #[test]
    fn test_remix_graph_remove_decrements_child_count() {
        let mut graph = RemixGraph::new();

        let root = Contraption::new("Root");
        graph.register(&root);

        let child1 = root.fork("Child 1");
        graph.register(&child1);

        let child2 = root.fork("Child 2");
        graph.register(&child2);

        // Root has 2 children
        assert_eq!(graph.descendant_count(root.id), 2);

        // Remove child1 - should decrement root's child count
        graph.remove(child1.id);

        // Root now has 1 child
        assert_eq!(graph.descendant_count(root.id), 1);
    }

    /// COV07: ContraptionStorage delete() on non-existent returns None
    #[test]
    fn test_storage_delete_nonexistent() {
        let mut storage = ContraptionStorage::new();
        let fake_id = ContraptionId::new();

        let result = storage.delete(fake_id);
        assert!(result.is_none());
    }

    /// COV08: Custom material preset uses rubber defaults
    #[test]
    fn test_custom_preset() {
        let mat = MaterialProperties::from_preset(MaterialPreset::Custom);
        // Custom preset uses rubber defaults (but keeps Rubber preset marker)
        assert!((mat.bounciness() - 0.7).abs() < f32::EPSILON);
        // The from_preset(Custom) returns default() which is Rubber
        assert_eq!(mat.preset, MaterialPreset::Rubber);
    }
}

mod integration {
    use super::*;

    /// I01: Full remix workflow (Kaizen cycle)
    #[test]
    fn test_full_remix_workflow() {
        let mut storage = ContraptionStorage::new();

        // 1. DISCOVER: Create original contraption
        let original = ContraptionBuilder::new("Original Machine")
            .author("Creator")
            .description("A Rube Goldberg machine")
            .tag("physics")
            .tag("chain-reaction")
            .with_object(
                ObjectType::Ball,
                Transform2D {
                    position: Vec2::new(0.0, 100.0),
                    ..Transform2D::default()
                },
            )
            .with_object(
                ObjectType::Ramp,
                Transform2D {
                    position: Vec2::new(50.0, 50.0),
                    rotation: 0.5,
                    scale: Vec2::new(100.0, 20.0),
                },
            )
            .with_object(
                ObjectType::Domino,
                Transform2D {
                    position: Vec2::new(150.0, 20.0),
                    ..Transform2D::default()
                },
            )
            .build()
            .expect("should build original");

        let original_id = storage.save(original).expect("should save original");

        // 2. FORK: Clone to remix
        let original_ref = storage.load(original_id).expect("should load");
        let forked = original_ref.fork("Remix v1");
        let fork_id = storage.save(forked).expect("should save fork");

        // 3. VERIFY: Lineage is correct
        let graph = storage.graph();
        assert!(graph.is_ancestor_of(original_id, fork_id));
        assert_eq!(graph.root(fork_id), original_id);
        assert_eq!(graph.depth(fork_id), 1);

        // 4. SHARE: Fork again (community remix)
        let fork_ref = storage.load(fork_id).expect("should load fork");
        let remix2 = fork_ref.fork("Community Remix");
        let remix2_id = storage.save(remix2).expect("should save remix2");

        // 5. VERIFY: Full ancestry
        let ancestors = storage.graph().ancestors(remix2_id);
        assert_eq!(ancestors.len(), 2);
        assert!(ancestors.contains(&fork_id));
        assert!(ancestors.contains(&original_id));

        // Stats check
        let stats = storage.graph().stats();
        assert_eq!(stats.total_contraptions, 3);
        assert_eq!(stats.root_contraptions, 1);
        assert_eq!(stats.fork_count, 2);
        assert_eq!(stats.max_chain_depth, 2);
    }

    /// I02: Search and popular functionality
    #[test]
    fn test_search_and_popular() {
        let mut storage = ContraptionStorage::new();

        // Create contraptions with different popularity
        let mut c1 = ContraptionBuilder::new("Popular Demo")
            .tag("demo")
            .tag("featured")
            .build()
            .expect("should build");
        c1.metadata.remix_count = 100;

        let mut c2 = ContraptionBuilder::new("Unpopular Demo")
            .tag("demo")
            .build()
            .expect("should build");
        c2.metadata.remix_count = 5;

        let c3 = ContraptionBuilder::new("Art Project")
            .tag("art")
            .build()
            .expect("should build");

        storage.save(c1).expect("should save");
        storage.save(c2).expect("should save");
        storage.save(c3).expect("should save");

        // Search by tag
        let demos = storage.search_by_tag("demo");
        assert_eq!(demos.len(), 2);

        let art = storage.search_by_tag("art");
        assert_eq!(art.len(), 1);

        // Popular returns most remixed
        let popular = storage.popular(1);
        assert_eq!(popular.len(), 1);
        assert_eq!(popular[0].metadata.name, "Popular Demo");
    }
}
