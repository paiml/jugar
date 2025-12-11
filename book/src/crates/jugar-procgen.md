# jugar-procgen

Procedural generation: noise, dungeons, and Wave Function Collapse.

## Value Noise

```rust
use jugar_procgen::noise::*;

let noise = ValueNoise::new(seed);

// Single value at point
let value = noise.sample(x, y);  // Returns 0.0 to 1.0

// With octaves (fractal noise)
let config = NoiseConfig {
    octaves: 4,
    persistence: 0.5,  // Amplitude decrease per octave
    lacunarity: 2.0,   // Frequency increase per octave
    scale: 0.01,
};
let value = noise.sample_octaves(x, y, &config);
```

## Noise Types

```rust
// Value noise (smooth)
let value = ValueNoise::new(seed);

// Perlin noise (gradient-based)
let perlin = PerlinNoise::new(seed);

// Simplex noise (faster, less artifacts)
let simplex = SimplexNoise::new(seed);

// Worley noise (cellular)
let worley = WorleyNoise::new(seed);
```

## Generating Terrain

```rust
fn generate_heightmap(width: usize, height: usize, seed: u64) -> Vec<f32> {
    let noise = SimplexNoise::new(seed);
    let config = NoiseConfig::default();

    let mut heightmap = vec![0.0; width * height];

    for y in 0..height {
        for x in 0..width {
            let value = noise.sample_octaves(
                x as f32,
                y as f32,
                &config,
            );
            heightmap[y * width + x] = value;
        }
    }

    heightmap
}
```

## Dungeon Generation

```rust
use jugar_procgen::dungeon::*;

// BSP-based dungeon
let config = DungeonConfig {
    width: 80,
    height: 60,
    min_room_size: 5,
    max_room_size: 15,
    corridor_width: 2,
};

let dungeon = BspDungeon::generate(config, seed);

// Access tiles
for y in 0..dungeon.height {
    for x in 0..dungeon.width {
        match dungeon.get_tile(x, y) {
            Tile::Floor => { /* walkable */ }
            Tile::Wall => { /* blocked */ }
            Tile::Door => { /* door */ }
        }
    }
}

// Get rooms
for room in dungeon.rooms() {
    spawn_enemies_in_room(room);
}
```

## Wave Function Collapse

```rust
use jugar_procgen::wfc::*;

// Define tiles and rules
let tiles = vec![
    Tile::new("grass").edges(["g", "g", "g", "g"]),
    Tile::new("water").edges(["w", "w", "w", "w"]),
    Tile::new("shore_n").edges(["g", "g", "w", "g"]),
    // ... more tiles with edge constraints
];

let wfc = WaveFunctionCollapse::new(tiles);

// Generate map
let result = wfc.generate(width, height, seed);

for y in 0..height {
    for x in 0..width {
        let tile = result.get(x, y);
        render_tile(tile, x, y);
    }
}
```

## WFC Constraints

```rust
// Socket-based constraints
let tiles = vec![
    Tile::new("road_horizontal")
        .sockets(Socket {
            north: "grass",
            south: "grass",
            east: "road",
            west: "road",
        }),
    Tile::new("road_vertical")
        .sockets(Socket {
            north: "road",
            south: "road",
            east: "grass",
            west: "grass",
        }),
    // ...
];
```

## Room Placement

```rust
use jugar_procgen::rooms::*;

let mut placer = RoomPlacer::new(100, 100);

// Add rooms with constraints
placer.add_room(Room::new(10, 10).tag("spawn"));
placer.add_room(Room::new(15, 15).tag("boss"));
placer.add_rooms(5, Room::random(5..10, 5..10).tag("normal"));

// Connect rooms
placer.connect_all_rooms();

// Generate
let layout = placer.generate(seed);
```

## Combining Techniques

```rust
fn generate_world(seed: u64) -> World {
    // 1. Generate heightmap with noise
    let heightmap = generate_heightmap(256, 256, seed);

    // 2. Generate dungeon layout
    let dungeon = BspDungeon::generate(config, seed);

    // 3. Use WFC to fill in details
    let details = wfc.generate(dungeon.width, dungeon.height, seed);

    // 4. Combine everything
    // ...
}
```
