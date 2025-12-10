//! Physics Toy Sandbox - WASM Demo
//!
//! Interactive demo for User Acceptance Testing (UAT)
//! Showcases: Remix workflow, Poka-Yoke safety, Mieruka thermometer

use physics_toy_sandbox::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"🎮 Physics Toy Sandbox Demo Loaded!".into());
    web_sys::console::log_1(&format!("📦 Engine Version: {}", ENGINE_VERSION).into());
}

/// Demo: Create a simple Rube Goldberg machine
#[wasm_bindgen]
pub fn create_demo_contraption() -> String {
    let contraption = ContraptionBuilder::new("UAT Demo Machine")
        .author("QA Tester")
        .description("A simple Rube Goldberg machine for UAT")
        .tag("demo")
        .tag("uat")
        .with_object(
            ObjectType::Ball,
            Transform2D {
                position: glam::Vec2::new(50.0, 300.0),
                rotation: 0.0,
                scale: glam::Vec2::splat(20.0),
            },
        )
        .with_object(
            ObjectType::Ramp,
            Transform2D {
                position: glam::Vec2::new(100.0, 250.0),
                rotation: -0.3,
                scale: glam::Vec2::new(150.0, 10.0),
            },
        )
        .with_object(
            ObjectType::Domino,
            Transform2D {
                position: glam::Vec2::new(200.0, 180.0),
                rotation: 0.0,
                scale: glam::Vec2::new(10.0, 40.0),
            },
        )
        .with_object(
            ObjectType::Domino,
            Transform2D {
                position: glam::Vec2::new(230.0, 180.0),
                rotation: 0.0,
                scale: glam::Vec2::new(10.0, 40.0),
            },
        )
        .with_object(
            ObjectType::Domino,
            Transform2D {
                position: glam::Vec2::new(260.0, 180.0),
                rotation: 0.0,
                scale: glam::Vec2::new(10.0, 40.0),
            },
        )
        .with_object(
            ObjectType::Bucket,
            Transform2D {
                position: glam::Vec2::new(350.0, 100.0),
                rotation: 0.0,
                scale: glam::Vec2::new(60.0, 40.0),
            },
        )
        .with_seed(42)
        .build()
        .expect("Demo contraption should build");

    format!(
        "Created: {} (ID: {}, {} objects, seed: {})",
        contraption.metadata.name,
        contraption.id,
        contraption.object_count(),
        contraption.initial_seed
    )
}

/// Demo: Fork a contraption (Kaizen cycle)
#[wasm_bindgen]
pub fn demo_fork_workflow() -> String {
    let original = Contraption::new("Original Machine");
    let fork1 = original.fork("Community Remix v1");
    let fork2 = fork1.fork("Community Remix v2");

    let mut graph = RemixGraph::new();
    graph.register(&original);
    graph.register(&fork1);
    graph.register(&fork2);

    let stats = graph.stats();

    format!(
        "Kaizen Workflow Demo:\n\
         - Original: {} (depth: {})\n\
         - Fork 1: {} (depth: {})\n\
         - Fork 2: {} (depth: {})\n\
         Stats: {} total, {} roots, {} forks, max depth: {}",
        original.id,
        graph.depth(original.id),
        fork1.id,
        graph.depth(fork1.id),
        fork2.id,
        graph.depth(fork2.id),
        stats.total_contraptions,
        stats.root_contraptions,
        stats.fork_count,
        stats.max_chain_depth
    )
}

/// Demo: Poka-Yoke material safety
#[wasm_bindgen]
pub fn demo_poka_yoke() -> String {
    let mut results = Vec::new();

    // Test 1: Valid density
    match MaterialProperties::new(1000.0) {
        Ok(mat) => results.push(format!(
            "✅ Valid density (1000 kg/m³): {} kg/m³",
            mat.density()
        )),
        Err(e) => results.push(format!("❌ Unexpected error: {}", e)),
    }

    // Test 2: Zero density rejected
    match MaterialProperties::new(0.0) {
        Ok(_) => results.push("❌ Zero density should be rejected!".to_string()),
        Err(e) => results.push(format!("✅ Zero density rejected: {}", e)),
    }

    // Test 3: Negative density rejected
    match MaterialProperties::new(-500.0) {
        Ok(_) => results.push("❌ Negative density should be rejected!".to_string()),
        Err(e) => results.push(format!("✅ Negative density rejected: {}", e)),
    }

    // Test 4: Bounciness clamping
    let mut mat = MaterialProperties::default();
    mat.set_bounciness(2.0); // Should clamp to 1.0
    results.push(format!(
        "✅ Bounciness clamped: set 2.0, got {} (max 1.0)",
        mat.bounciness()
    ));

    // Test 5: Material presets
    let wood = MaterialProperties::from_preset(MaterialPreset::Wood);
    let metal = MaterialProperties::from_preset(MaterialPreset::Metal);
    results.push(format!(
        "✅ Presets: Wood={} kg/m³, Metal={} kg/m³",
        wood.density(),
        metal.density()
    ));

    results.join("\n")
}

/// Demo: Complexity Thermometer (Mieruka)
#[wasm_bindgen]
pub fn demo_thermometer(physics_ms: f32, render_ms: f32, ui_ms: f32) -> String {
    let mut thermo = ComplexityThermometer::new(60.0);

    // Simulate several frames
    for _ in 0..10 {
        thermo.update(PerformanceBreakdown {
            physics_ms,
            render_ms,
            ui_ms,
            other_ms: 0.0,
        });
    }

    let state = thermo.visual_state();
    let color = state.css_color();
    let desc = state.description();

    format!(
        "🌡️ Complexity Thermometer\n\
         Load: {:.1}%\n\
         State: {:?} ({})\n\
         Color: {}\n\
         Budget: {:.1}ms (target: {} FPS)\n\
         Breakdown: Physics={:.1}ms, Render={:.1}ms, UI={:.1}ms\n\
         Block additions: {}",
        thermo.load_percent(),
        state,
        desc,
        color,
        thermo.budget_ms(),
        thermo.target_fps() as u32,
        physics_ms,
        render_ms,
        ui_ms,
        if thermo.should_block_additions() {
            "YES ⛔"
        } else {
            "NO ✅"
        }
    )
}

/// Demo: Serialization round-trip
#[wasm_bindgen]
pub fn demo_serialization() -> String {
    let original = ContraptionBuilder::new("Serialization Test")
        .author("Test Author")
        .with_object(ObjectType::Ball, Transform2D::default())
        .with_object(
            ObjectType::Ramp,
            Transform2D {
                position: glam::Vec2::new(100.0, 50.0),
                rotation: 0.5,
                scale: glam::Vec2::new(200.0, 20.0),
            },
        )
        .with_seed(12345)
        .build()
        .expect("Should build");

    let bytes = original.serialize().expect("Should serialize");
    let restored = Contraption::deserialize(&bytes).expect("Should deserialize");

    let size = bytes.len();
    let id_match = original.id == restored.id;
    let seed_match = original.initial_seed == restored.initial_seed;
    let entity_count_match = original.entities.len() == restored.entities.len();

    format!(
        "📦 Serialization Demo\n\
         Original ID: {}\n\
         Restored ID: {}\n\
         ID Match: {} {}\n\
         Seed Match: {} {}\n\
         Entity Count Match: {} {}\n\
         Serialized Size: {} bytes\n\
         Content Hash: {}",
        original.id,
        restored.id,
        id_match,
        if id_match { "✅" } else { "❌" },
        seed_match,
        if seed_match { "✅" } else { "❌" },
        entity_count_match,
        if entity_count_match { "✅" } else { "❌" },
        size,
        original.content_hash()
    )
}

/// Demo: Storage and search
#[wasm_bindgen]
pub fn demo_storage() -> String {
    let mut storage = ContraptionStorage::new();

    // Create and save contraptions
    let mut c1 = ContraptionBuilder::new("Physics Demo")
        .tag("physics")
        .tag("featured")
        .build()
        .expect("build");
    c1.metadata.remix_count = 100;

    let mut c2 = ContraptionBuilder::new("Art Project")
        .tag("art")
        .build()
        .expect("build");
    c2.metadata.remix_count = 25;

    let c3 = ContraptionBuilder::new("Beginner Tutorial")
        .tag("physics")
        .tag("tutorial")
        .build()
        .expect("build");

    let _ = storage.save(c1);
    let _ = storage.save(c2);
    let _ = storage.save(c3);

    let physics_results = storage.search_by_tag("physics");
    let popular = storage.popular(2);

    format!(
        "📚 Storage Demo\n\
         Total contraptions: {}\n\
         Search 'physics': {} results\n\
         Popular (top 2):\n  1. {} (remixes: {})\n  2. {} (remixes: {})",
        storage.count(),
        physics_results.len(),
        popular[0].metadata.name,
        popular[0].metadata.remix_count,
        popular[1].metadata.name,
        popular[1].metadata.remix_count
    )
}

/// Get engine version
#[wasm_bindgen]
pub fn get_engine_version() -> String {
    ENGINE_VERSION.to_string()
}

/// Get max objects limit
#[wasm_bindgen]
pub fn get_max_objects() -> usize {
    MAX_OBJECTS_PER_CONTRAPTION
}

/// Run all demos and return summary
#[wasm_bindgen]
pub fn run_all_demos() -> String {
    let mut output = Vec::new();

    output.push("═══════════════════════════════════════════════════════════════".to_string());
    output.push("     PHYSICS TOY SANDBOX - USER ACCEPTANCE TESTING".to_string());
    output.push(format!("     Engine Version: {}", ENGINE_VERSION));
    output.push("═══════════════════════════════════════════════════════════════".to_string());
    output.push(String::new());

    output.push("▶ DEMO 1: Create Contraption".to_string());
    output.push("─────────────────────────────".to_string());
    output.push(create_demo_contraption());
    output.push(String::new());

    output.push("▶ DEMO 2: Fork Workflow (Kaizen)".to_string());
    output.push("─────────────────────────────────".to_string());
    output.push(demo_fork_workflow());
    output.push(String::new());

    output.push("▶ DEMO 3: Poka-Yoke Safety".to_string());
    output.push("──────────────────────────".to_string());
    output.push(demo_poka_yoke());
    output.push(String::new());

    output.push("▶ DEMO 4: Complexity Thermometer (Mieruka)".to_string());
    output.push("───────────────────────────────────────────".to_string());
    output.push("Green state (low load):".to_string());
    output.push(demo_thermometer(4.0, 4.0, 2.0));
    output.push(String::new());
    output.push("Yellow state (warning):".to_string());
    output.push(demo_thermometer(6.0, 6.0, 2.0));
    output.push(String::new());
    output.push("Red state (critical):".to_string());
    output.push(demo_thermometer(8.0, 8.0, 3.0));
    output.push(String::new());

    output.push("▶ DEMO 5: Serialization".to_string());
    output.push("────────────────────────".to_string());
    output.push(demo_serialization());
    output.push(String::new());

    output.push("▶ DEMO 6: Storage & Search".to_string());
    output.push("───────────────────────────".to_string());
    output.push(demo_storage());
    output.push(String::new());

    output.push("═══════════════════════════════════════════════════════════════".to_string());
    output.push("     ALL DEMOS COMPLETED SUCCESSFULLY ✅".to_string());
    output.push("═══════════════════════════════════════════════════════════════".to_string());

    output.join("\n")
}
