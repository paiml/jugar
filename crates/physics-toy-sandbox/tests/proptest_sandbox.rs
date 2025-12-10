//! Property-Based Tests for Physics Toy Sandbox
//!
//! Uses proptest to verify invariants across random inputs.
//! Toyota Way: Poka-Yoke through property verification.

#![allow(
    unused_results,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::uninlined_format_args,
    clippy::doc_markdown,
    clippy::redundant_clone,
    clippy::std_instead_of_core,
    clippy::unreadable_literal
)]

use glam::Vec2;
use physics_toy_sandbox::*;
use proptest::prelude::*;

// =============================================================================
// ARBITRARY IMPLEMENTATIONS
// =============================================================================

/// Generate arbitrary Vec2 within reasonable bounds
fn arb_vec2() -> impl Strategy<Value = Vec2> {
    (-10000.0f32..10000.0, -10000.0f32..10000.0).prop_map(|(x, y)| Vec2::new(x, y))
}

/// Generate arbitrary rotation (in radians)
fn arb_rotation() -> impl Strategy<Value = f32> {
    -std::f32::consts::TAU..std::f32::consts::TAU
}

/// Generate arbitrary scale (positive only)
fn arb_scale() -> impl Strategy<Value = Vec2> {
    (0.1f32..100.0, 0.1f32..100.0).prop_map(|(x, y)| Vec2::new(x, y))
}

/// Generate arbitrary Transform2D
fn arb_transform() -> impl Strategy<Value = Transform2D> {
    (arb_vec2(), arb_rotation(), arb_scale()).prop_map(|(position, rotation, scale)| Transform2D {
        position,
        rotation,
        scale,
    })
}

/// Generate arbitrary ObjectType
fn arb_object_type() -> impl Strategy<Value = ObjectType> {
    prop_oneof![
        Just(ObjectType::Ball),
        Just(ObjectType::Domino),
        Just(ObjectType::Ramp),
        Just(ObjectType::Lever),
        Just(ObjectType::Pulley),
        Just(ObjectType::Spring),
        Just(ObjectType::Fan),
        Just(ObjectType::Magnet),
        Just(ObjectType::Bucket),
        Just(ObjectType::Sensor),
    ]
}

/// Generate arbitrary MaterialPreset
fn arb_material_preset() -> impl Strategy<Value = MaterialPreset> {
    prop_oneof![
        Just(MaterialPreset::Wood),
        Just(MaterialPreset::Metal),
        Just(MaterialPreset::Rubber),
        Just(MaterialPreset::Ice),
        Just(MaterialPreset::Custom),
    ]
}

/// Generate arbitrary valid density (kg/m³)
fn arb_density() -> impl Strategy<Value = f32> {
    0.001f32..100000.0 // From nearly zero to 100,000 kg/m³
}

/// Generate arbitrary bounciness
fn arb_bounciness() -> impl Strategy<Value = f32> {
    -1.0f32..2.0 // Test clamping behavior
}

/// Generate arbitrary friction
fn arb_friction() -> impl Strategy<Value = f32> {
    -1.0f32..5.0 // Test clamping behavior
}

// =============================================================================
// PROPERTY TESTS: MATERIAL INVARIANTS
// =============================================================================

proptest! {
    /// Property: Valid density always creates valid material
    #[test]
    fn prop_valid_density_creates_material(density in arb_density()) {
        let result = MaterialProperties::new(density);
        prop_assert!(result.is_ok(), "Valid density {} should create material", density);

        let mat = result.unwrap();
        prop_assert!(mat.density() > 0.0, "Density should be positive");
    }

    /// Property: Zero or negative density always fails
    #[test]
    fn prop_invalid_density_rejected(density in -100000.0f32..=0.0) {
        let result = MaterialProperties::new(density);
        prop_assert!(result.is_err(), "Invalid density {} should be rejected", density);
    }

    /// Property: Bounciness is always clamped to [0, 1]
    #[test]
    fn prop_bounciness_always_clamped(bounciness in arb_bounciness()) {
        let mut mat = MaterialProperties::default();
        mat.set_bounciness(bounciness);

        prop_assert!(mat.bounciness() >= 0.0, "Bounciness should be >= 0");
        prop_assert!(mat.bounciness() <= 1.0, "Bounciness should be <= 1");
    }

    /// Property: Friction is always non-negative
    #[test]
    fn prop_friction_always_non_negative(
        static_f in arb_friction(),
        dynamic_f in arb_friction()
    ) {
        let mut mat = MaterialProperties::default();
        mat.set_friction_static(static_f);
        mat.set_friction_dynamic(dynamic_f);

        prop_assert!(mat.friction_static() >= 0.0, "Static friction should be >= 0");
        prop_assert!(mat.friction_dynamic() >= 0.0, "Dynamic friction should be >= 0");
    }

    /// Property: Mass is always non-negative for any volume
    #[test]
    fn prop_mass_non_negative(volume in 0.0f32..1000.0) {
        let mat = MaterialProperties::default();
        let mass = mat.mass_for_volume(volume);

        prop_assert!(mass >= 0.0, "Mass should be >= 0 for volume {}", volume);
        prop_assert!(!mass.is_nan(), "Mass should not be NaN");
        prop_assert!(!mass.is_infinite(), "Mass should not be infinite");
    }

    /// Property: Material presets always have valid properties
    #[test]
    fn prop_preset_always_valid(preset in arb_material_preset()) {
        let mat = MaterialProperties::from_preset(preset);

        prop_assert!(mat.density() > 0.0, "Preset density should be positive");
        prop_assert!(mat.bounciness() >= 0.0, "Preset bounciness should be >= 0");
        prop_assert!(mat.bounciness() <= 1.0, "Preset bounciness should be <= 1");
        prop_assert!(mat.friction_static() >= 0.0, "Preset static friction should be >= 0");
        prop_assert!(mat.friction_dynamic() >= 0.0, "Preset dynamic friction should be >= 0");
    }
}

// =============================================================================
// PROPERTY TESTS: TRANSFORM INVARIANTS
// =============================================================================

proptest! {
    /// Property: Transform serializes and deserializes without loss
    #[test]
    fn prop_transform_roundtrip(transform in arb_transform()) {
        let json = serde_json::to_string(&transform).unwrap();
        let restored: Transform2D = serde_json::from_str(&json).unwrap();

        prop_assert!((transform.position.x - restored.position.x).abs() < f32::EPSILON);
        prop_assert!((transform.position.y - restored.position.y).abs() < f32::EPSILON);
        prop_assert!((transform.rotation - restored.rotation).abs() < f32::EPSILON);
        prop_assert!((transform.scale.x - restored.scale.x).abs() < f32::EPSILON);
        prop_assert!((transform.scale.y - restored.scale.y).abs() < f32::EPSILON);
    }
}

// =============================================================================
// PROPERTY TESTS: CONTRAPTION INVARIANTS
// =============================================================================

proptest! {
    /// Property: Contraption IDs are always unique
    #[test]
    fn prop_contraption_ids_unique(
        name1 in "[a-zA-Z ]{1,20}",
        name2 in "[a-zA-Z ]{1,20}"
    ) {
        let c1 = Contraption::new(&name1);
        let c2 = Contraption::new(&name2);

        prop_assert_ne!(c1.id, c2.id, "Contraption IDs should be unique");
    }

    /// Property: Fork always references parent
    #[test]
    fn prop_fork_references_parent(name in "[a-zA-Z ]{1,20}") {
        let parent = Contraption::new(&name);
        let parent_id = parent.id;
        let child = parent.fork("Fork");

        prop_assert_eq!(child.forked_from, Some(parent_id));
        prop_assert!(child.is_fork());
        prop_assert!(!parent.is_fork());
    }

    /// Property: Content hash is consistent
    #[test]
    fn prop_content_hash_consistent(seed in 0u64..u64::MAX) {
        let c = ContraptionBuilder::new("Hash Test")
            .with_seed(seed)
            .build()
            .unwrap();

        let hash1 = c.content_hash();
        let hash2 = c.content_hash();

        prop_assert_eq!(hash1, hash2, "Content hash should be consistent");
    }

    /// Property: Serialization round-trip preserves identity
    #[test]
    fn prop_serialization_preserves_id(name in "[a-zA-Z ]{1,20}", seed in 0u64..u64::MAX) {
        let original = ContraptionBuilder::new(&name)
            .with_seed(seed)
            .build()
            .unwrap();

        let bytes = original.serialize().unwrap();
        let restored = Contraption::deserialize(&bytes).unwrap();

        prop_assert_eq!(original.id, restored.id);
        prop_assert_eq!(original.initial_seed, restored.initial_seed);
        prop_assert_eq!(original.engine_version, restored.engine_version);
    }
}

// =============================================================================
// PROPERTY TESTS: OBJECT TYPE INVARIANTS
// =============================================================================

proptest! {
    /// Property: Object type categories are mutually exclusive
    #[test]
    fn prop_object_type_mutually_exclusive(obj_type in arb_object_type()) {
        let is_dynamic = obj_type.is_dynamic();
        let is_trigger = obj_type.is_trigger();
        let is_constraint = obj_type.is_constraint();

        // Count how many categories this object belongs to
        let category_count = [is_dynamic, is_trigger, is_constraint]
            .iter()
            .filter(|&&x| x)
            .count();

        // An object can be at most one category (or none for static objects like Ramp)
        prop_assert!(category_count <= 1,
            "Object {:?} should be in at most one category, but is in {}",
            obj_type, category_count);
    }

    /// Property: Entity gets material only for non-trigger objects
    #[test]
    fn prop_entity_material_for_physics_objects(obj_type in arb_object_type()) {
        let entity = SerializedEntity::new(obj_type, Transform2D::default());

        if obj_type.is_trigger() {
            prop_assert!(entity.material.is_none(),
                "Trigger {:?} should not have material", obj_type);
        } else {
            prop_assert!(entity.material.is_some(),
                "Non-trigger {:?} should have material", obj_type);
        }
    }
}

// =============================================================================
// PROPERTY TESTS: STORAGE INVARIANTS
// =============================================================================

proptest! {
    /// Property: Storage count increases with saves
    #[test]
    fn prop_storage_count_increases(num_saves in 1usize..10) {
        let mut storage = ContraptionStorage::new();

        for i in 0..num_saves {
            let c = Contraption::new(format!("Test {}", i));
            storage.save(c).unwrap();
            prop_assert_eq!(storage.count(), i + 1);
        }
    }

    /// Property: Saved contraption can always be loaded
    #[test]
    fn prop_save_then_load(name in "[a-zA-Z ]{1,20}") {
        let mut storage = ContraptionStorage::new();
        let c = Contraption::new(&name);
        let id = c.id;

        storage.save(c).unwrap();
        let loaded = storage.load(id);

        prop_assert!(loaded.is_ok(), "Saved contraption should be loadable");
        prop_assert_eq!(&loaded.unwrap().metadata.name, &name);
    }
}

// =============================================================================
// PROPERTY TESTS: REMIX GRAPH INVARIANTS
// =============================================================================

proptest! {
    /// Property: Root has depth 0
    #[test]
    fn prop_root_depth_zero(name in "[a-zA-Z ]{1,20}") {
        let mut graph = RemixGraph::new();
        let root = Contraption::new(&name);
        graph.register(&root);

        prop_assert_eq!(graph.depth(root.id), 0);
        prop_assert!(graph.is_root(root.id));
    }

    /// Property: Fork depth is parent depth + 1
    #[test]
    fn prop_fork_depth_increment(name in "[a-zA-Z ]{1,20}") {
        let mut graph = RemixGraph::new();

        let root = Contraption::new(&name);
        graph.register(&root);

        let fork = root.fork("Fork");
        graph.register(&fork);

        prop_assert_eq!(graph.depth(fork.id), graph.depth(root.id) + 1);
    }

    /// Property: Ancestors list is in correct order (immediate parent first)
    #[test]
    fn prop_ancestors_order(name in "[a-zA-Z ]{1,20}") {
        let mut graph = RemixGraph::new();

        let root = Contraption::new(&name);
        graph.register(&root);

        let child1 = root.fork("Child 1");
        graph.register(&child1);

        let child2 = child1.fork("Child 2");
        graph.register(&child2);

        let ancestors = graph.ancestors(child2.id);

        // First ancestor should be immediate parent
        prop_assert_eq!(ancestors.first().copied(), Some(child1.id));
        // Last ancestor should be root
        prop_assert_eq!(ancestors.last().copied(), Some(root.id));
    }
}

// =============================================================================
// PROPERTY TESTS: THERMOMETER INVARIANTS
// =============================================================================

proptest! {
    /// Property: Load factor is never negative
    #[test]
    fn prop_load_factor_non_negative(
        physics_ms in 0.0f32..100.0,
        render_ms in 0.0f32..100.0,
        ui_ms in 0.0f32..100.0
    ) {
        let mut thermo = ComplexityThermometer::default();
        thermo.update(PerformanceBreakdown {
            physics_ms,
            render_ms,
            ui_ms,
            other_ms: 0.0,
        });

        prop_assert!(thermo.load() >= 0.0, "Load should be >= 0");
        prop_assert!(!thermo.load().is_nan(), "Load should not be NaN");
    }

    /// Property: Visual state is always valid
    #[test]
    fn prop_visual_state_valid(load in 0.0f32..2.0) {
        let mut thermo = ComplexityThermometer::default();
        // Simulate to set load
        for _ in 0..60 {
            thermo.update(PerformanceBreakdown {
                physics_ms: load * 16.67 / 3.0,
                render_ms: load * 16.67 / 3.0,
                ui_ms: load * 16.67 / 3.0,
                other_ms: 0.0,
            });
        }

        let state = thermo.visual_state();
        prop_assert!(
            matches!(state, ThermometerState::Green | ThermometerState::Yellow | ThermometerState::Red),
            "Visual state should be valid enum variant"
        );
    }

    /// Property: Block threshold is consistent with visual state
    #[test]
    fn prop_block_consistent_with_state(
        physics_ms in 0.0f32..30.0,
        render_ms in 0.0f32..30.0,
        ui_ms in 0.0f32..10.0
    ) {
        let mut thermo = ComplexityThermometer::default();
        for _ in 0..60 {
            thermo.update(PerformanceBreakdown {
                physics_ms,
                render_ms,
                ui_ms,
                other_ms: 0.0,
            });
        }

        let state = thermo.visual_state();
        let should_block = thermo.should_block_additions();

        // If red, should block
        if state == ThermometerState::Red {
            prop_assert!(should_block, "Red state should block additions");
        }

        // If green, should not block
        if state == ThermometerState::Green {
            prop_assert!(!should_block, "Green state should not block additions");
        }
    }
}
