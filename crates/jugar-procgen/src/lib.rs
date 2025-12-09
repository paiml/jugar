//! # jugar-procgen
//!
//! Procedural generation for Jugar including noise, dungeon generation, and WFC.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Procedural generation errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ProcgenError {
    /// Generation failed to complete
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    /// WFC contradiction (no valid tile placement)
    #[error("WFC contradiction at ({x}, {y})")]
    WfcContradiction {
        /// X coordinate
        x: usize,
        /// Y coordinate
        y: usize,
    },
}

/// Result type for procgen operations
pub type Result<T> = core::result::Result<T, ProcgenError>;

// ============================================================================
// NOISE GENERATION
// ============================================================================

/// Simple seeded random number generator (xorshift)
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a new RNG with a seed
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generates the next random u64
    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Generates a random f32 in [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32)
    }

    /// Generates a random f32 in [min, max)
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Generates a random usize in [0, max)
    pub fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u64() as usize) % max
    }

    /// Generates a random i32 in [min, max)
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next_usize((max - min) as usize) as i32)
    }

    /// Shuffles a slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.next_usize(i + 1);
            slice.swap(i, j);
        }
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(12345)
    }
}

/// Value noise generator
#[derive(Debug, Clone)]
pub struct ValueNoise {
    seed: u64,
    scale: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
}

impl ValueNoise {
    /// Creates a new value noise generator
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            scale: 1.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    /// Sets the scale
    #[must_use]
    pub const fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the number of octaves
    #[must_use]
    pub const fn with_octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves;
        self
    }

    /// Sets persistence (amplitude multiplier per octave)
    #[must_use]
    pub const fn with_persistence(mut self, persistence: f32) -> Self {
        self.persistence = persistence;
        self
    }

    /// Sets lacunarity (frequency multiplier per octave)
    #[must_use]
    pub const fn with_lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = lacunarity;
        self
    }

    /// Samples 2D noise at a point
    #[must_use]
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0 / self.scale;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += self.raw_noise(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        total / max_value
    }

    fn raw_noise(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - x.floor();
        let yf = y - y.floor();

        // Smoothstep
        let u = xf * xf * (3.0 - 2.0 * xf);
        let v = yf * yf * (3.0 - 2.0 * yf);

        // Corner values
        let aa = self.hash(xi, yi);
        let ba = self.hash(xi + 1, yi);
        let ab = self.hash(xi, yi + 1);
        let bb = self.hash(xi + 1, yi + 1);

        // Bilinear interpolation
        let x1 = lerp(aa, ba, u);
        let x2 = lerp(ab, bb, u);
        lerp(x1, x2, v)
    }

    fn hash(&self, x: i32, y: i32) -> f32 {
        let n = (x.wrapping_mul(374761393))
            .wrapping_add(y.wrapping_mul(668265263))
            .wrapping_add(self.seed as i32);
        let n = n.wrapping_mul(n.wrapping_mul(n.wrapping_mul(60493))).wrapping_add(19990303);
        let n = (n >> 1) & 0x7FFF_FFFF;
        n as f32 / 0x7FFF_FFFF as f32
    }
}

impl Default for ValueNoise {
    fn default() -> Self {
        Self::new(0)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

// ============================================================================
// DUNGEON GENERATION
// ============================================================================

/// Dungeon tile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DungeonTile {
    /// Solid wall
    #[default]
    Wall,
    /// Floor (walkable)
    Floor,
    /// Door
    Door,
    /// Corridor floor
    Corridor,
}

impl DungeonTile {
    /// Returns true if the tile is walkable
    #[must_use]
    pub const fn is_walkable(self) -> bool {
        matches!(self, Self::Floor | Self::Door | Self::Corridor)
    }
}

/// A room in the dungeon
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

impl Room {
    /// Creates a new room
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center of the room
    #[must_use]
    pub const fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Checks if this room intersects another
    #[must_use]
    pub const fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }
}

/// Generated dungeon
#[derive(Debug, Clone)]
pub struct Dungeon {
    /// Width in tiles
    pub width: usize,
    /// Height in tiles
    pub height: usize,
    /// Tile data (row-major)
    pub tiles: Vec<DungeonTile>,
    /// Rooms
    pub rooms: Vec<Room>,
}

impl Dungeon {
    /// Creates an empty dungeon filled with walls
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![DungeonTile::Wall; width * height],
            rooms: Vec::new(),
        }
    }

    /// Gets a tile at position
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<DungeonTile> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    /// Sets a tile at position
    pub fn set(&mut self, x: usize, y: usize, tile: DungeonTile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    /// Checks if a position is in bounds
    #[must_use]
    pub const fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    /// Returns walkable positions
    #[must_use]
    pub fn walkable_positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(tile) = self.get(x, y) {
                    if tile.is_walkable() {
                        positions.push((x, y));
                    }
                }
            }
        }
        positions
    }
}

/// Dungeon generator using BSP (Binary Space Partition)
#[derive(Debug, Clone)]
pub struct DungeonGenerator {
    /// Dungeon width
    pub width: usize,
    /// Dungeon height
    pub height: usize,
    /// Minimum room size
    pub min_room_size: i32,
    /// Maximum room size
    pub max_room_size: i32,
    /// Number of rooms to generate
    pub room_count: usize,
    /// Room padding (space between rooms)
    pub padding: i32,
}

impl DungeonGenerator {
    /// Creates a new dungeon generator
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            min_room_size: 4,
            max_room_size: 10,
            room_count: 10,
            padding: 1,
        }
    }

    /// Sets the room size range
    #[must_use]
    pub const fn with_room_size(mut self, min: i32, max: i32) -> Self {
        self.min_room_size = min;
        self.max_room_size = max;
        self
    }

    /// Sets the room count
    #[must_use]
    pub const fn with_room_count(mut self, count: usize) -> Self {
        self.room_count = count;
        self
    }

    /// Generates a dungeon with the given seed
    pub fn generate(&self, seed: u64) -> Result<Dungeon> {
        let mut rng = Rng::new(seed);
        let mut dungeon = Dungeon::new(self.width, self.height);

        // Generate rooms
        for _ in 0..self.room_count * 10 {
            if dungeon.rooms.len() >= self.room_count {
                break;
            }

            let w = rng.range_i32(self.min_room_size, self.max_room_size + 1);
            let h = rng.range_i32(self.min_room_size, self.max_room_size + 1);
            let x = rng.range_i32(self.padding, self.width as i32 - w - self.padding);
            let y = rng.range_i32(self.padding, self.height as i32 - h - self.padding);

            let room = Room::new(x, y, w, h);

            // Check for overlap with existing rooms
            let overlaps = dungeon.rooms.iter().any(|r| {
                let padded = Room::new(
                    r.x - self.padding,
                    r.y - self.padding,
                    r.width + self.padding * 2,
                    r.height + self.padding * 2,
                );
                room.intersects(&padded)
            });

            if !overlaps {
                // Carve out the room
                for ry in y..(y + h) {
                    for rx in x..(x + w) {
                        dungeon.set(rx as usize, ry as usize, DungeonTile::Floor);
                    }
                }
                dungeon.rooms.push(room);
            }
        }

        if dungeon.rooms.is_empty() {
            return Err(ProcgenError::GenerationFailed(
                "Could not place any rooms".to_string(),
            ));
        }

        // Connect rooms with corridors
        for i in 1..dungeon.rooms.len() {
            let (x1, y1) = dungeon.rooms[i - 1].center();
            let (x2, y2) = dungeon.rooms[i].center();

            // Randomly choose horizontal-first or vertical-first
            if rng.next_f32() < 0.5 {
                self.carve_h_corridor(&mut dungeon, x1, x2, y1);
                self.carve_v_corridor(&mut dungeon, y1, y2, x2);
            } else {
                self.carve_v_corridor(&mut dungeon, y1, y2, x1);
                self.carve_h_corridor(&mut dungeon, x1, x2, y2);
            }
        }

        Ok(dungeon)
    }

    fn carve_h_corridor(&self, dungeon: &mut Dungeon, x1: i32, x2: i32, y: i32) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in start..=end {
            if dungeon.in_bounds(x, y) {
                let tile = dungeon.get(x as usize, y as usize).unwrap_or(DungeonTile::Wall);
                if tile == DungeonTile::Wall {
                    dungeon.set(x as usize, y as usize, DungeonTile::Corridor);
                }
            }
        }
    }

    fn carve_v_corridor(&self, dungeon: &mut Dungeon, y1: i32, y2: i32, x: i32) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in start..=end {
            if dungeon.in_bounds(x, y) {
                let tile = dungeon.get(x as usize, y as usize).unwrap_or(DungeonTile::Wall);
                if tile == DungeonTile::Wall {
                    dungeon.set(x as usize, y as usize, DungeonTile::Corridor);
                }
            }
        }
    }
}

impl Default for DungeonGenerator {
    fn default() -> Self {
        Self::new(50, 50)
    }
}

// ============================================================================
// WAVE FUNCTION COLLAPSE (WFC)
// ============================================================================

/// Tile index for WFC
pub type TileId = u16;

/// Direction for adjacency rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Up (+Y)
    Up,
    /// Down (-Y)
    Down,
    /// Left (-X)
    Left,
    /// Right (+X)
    Right,
}

impl Direction {
    /// Returns the opposite direction
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Returns the delta for this direction
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    /// All directions
    pub const ALL: [Direction; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

/// Adjacency rules for WFC
#[derive(Debug, Clone, Default)]
pub struct AdjacencyRules {
    /// Allowed neighbors per tile per direction
    rules: Vec<[HashSet<TileId>; 4]>,
}

impl AdjacencyRules {
    /// Creates new adjacency rules for a tile count
    #[must_use]
    pub fn new(tile_count: usize) -> Self {
        Self {
            rules: vec![Default::default(); tile_count],
        }
    }

    /// Adds an adjacency rule
    pub fn add(&mut self, tile: TileId, direction: Direction, neighbor: TileId) {
        if (tile as usize) < self.rules.len() {
            let _ = self.rules[tile as usize][direction as usize].insert(neighbor);
        }
    }

    /// Gets allowed neighbors for a tile in a direction
    #[must_use]
    pub fn allowed(&self, tile: TileId, direction: Direction) -> Option<&HashSet<TileId>> {
        self.rules
            .get(tile as usize)
            .map(|r| &r[direction as usize])
    }
}

/// Cell in the WFC grid
#[derive(Debug, Clone)]
pub struct WfcCell {
    /// Possible tiles (entropy)
    pub possibilities: HashSet<TileId>,
    /// Collapsed tile (if any)
    pub collapsed: Option<TileId>,
}

impl WfcCell {
    /// Creates a new cell with all possibilities
    #[must_use]
    pub fn new(all_tiles: &[TileId]) -> Self {
        Self {
            possibilities: all_tiles.iter().copied().collect(),
            collapsed: None,
        }
    }

    /// Returns the entropy (number of possibilities)
    #[must_use]
    pub fn entropy(&self) -> usize {
        self.possibilities.len()
    }

    /// Checks if collapsed
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.collapsed.is_some()
    }
}

/// Wave Function Collapse generator
pub struct Wfc {
    width: usize,
    height: usize,
    cells: Vec<WfcCell>,
    rules: AdjacencyRules,
    all_tiles: Vec<TileId>,
    rng: Rng,
}

impl Wfc {
    /// Creates a new WFC generator
    #[must_use]
    pub fn new(width: usize, height: usize, tile_count: usize, seed: u64) -> Self {
        let all_tiles: Vec<TileId> = (0..tile_count as TileId).collect();
        let cells = vec![WfcCell::new(&all_tiles); width * height];

        Self {
            width,
            height,
            cells,
            rules: AdjacencyRules::new(tile_count),
            all_tiles,
            rng: Rng::new(seed),
        }
    }

    /// Gets the adjacency rules for modification
    pub fn rules_mut(&mut self) -> &mut AdjacencyRules {
        &mut self.rules
    }

    /// Gets a cell
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<&WfcCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Runs the WFC algorithm to completion
    pub fn collapse(&mut self) -> Result<()> {
        loop {
            // Find cell with lowest entropy (not collapsed)
            let min_cell = self.find_min_entropy_cell();

            let Some((x, y)) = min_cell else {
                // All cells collapsed
                return Ok(());
            };

            // Collapse this cell
            self.collapse_cell(x, y)?;

            // Propagate constraints
            self.propagate(x, y)?;
        }
    }

    /// Returns the collapsed grid
    #[must_use]
    pub fn result(&self) -> Vec<Option<TileId>> {
        self.cells.iter().map(|c| c.collapsed).collect()
    }

    fn find_min_entropy_cell(&self) -> Option<(usize, usize)> {
        let mut min_entropy = usize::MAX;
        let mut candidates = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y * self.width + x];
                if cell.is_collapsed() {
                    continue;
                }

                let entropy = cell.entropy();
                if entropy < min_entropy {
                    min_entropy = entropy;
                    candidates.clear();
                    candidates.push((x, y));
                } else if entropy == min_entropy {
                    candidates.push((x, y));
                }
            }
        }

        if candidates.is_empty() {
            None
        } else {
            let idx = self.rng.clone().next_usize(candidates.len());
            Some(candidates[idx])
        }
    }

    fn collapse_cell(&mut self, x: usize, y: usize) -> Result<()> {
        let cell = &mut self.cells[y * self.width + x];

        if cell.possibilities.is_empty() {
            return Err(ProcgenError::WfcContradiction { x, y });
        }

        // Pick a random possibility
        let possibilities: Vec<_> = cell.possibilities.iter().copied().collect();
        let idx = self.rng.next_usize(possibilities.len());
        let chosen = possibilities[idx];

        cell.collapsed = Some(chosen);
        cell.possibilities.clear();
        let _ = cell.possibilities.insert(chosen);

        Ok(())
    }

    fn propagate(&mut self, start_x: usize, start_y: usize) -> Result<()> {
        let mut stack = vec![(start_x, start_y)];

        while let Some((x, y)) = stack.pop() {
            let cell = &self.cells[y * self.width + x];
            let current_possibilities = cell.possibilities.clone();

            for dir in Direction::ALL {
                let (dx, dy) = dir.delta();
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;
                let neighbor = &mut self.cells[ny * self.width + nx];

                if neighbor.is_collapsed() {
                    continue;
                }

                // Calculate allowed tiles based on current cell's possibilities
                let mut allowed: HashSet<TileId> = HashSet::new();
                for &tile in &current_possibilities {
                    if let Some(neighbors) = self.rules.allowed(tile, dir) {
                        allowed.extend(neighbors);
                    }
                }

                // Intersect with neighbor's possibilities
                let old_len = neighbor.possibilities.len();
                neighbor.possibilities.retain(|t| allowed.contains(t));

                if neighbor.possibilities.is_empty() {
                    return Err(ProcgenError::WfcContradiction { x: nx, y: ny });
                }

                if neighbor.possibilities.len() < old_len {
                    stack.push((nx, ny));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Wfc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wfc")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("tiles", &self.all_tiles.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RNG tests
    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_rng_range_f32() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let val = rng.range_f32(10.0, 20.0);
            assert!(val >= 10.0 && val < 20.0);
        }
    }

    #[test]
    fn test_rng_shuffle() {
        let mut rng = Rng::new(42);
        let mut items = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut items);

        // Should be permutation (same elements)
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    // Noise tests
    #[test]
    fn test_value_noise_range() {
        let noise = ValueNoise::new(42).with_scale(10.0);

        for x in 0..10 {
            for y in 0..10 {
                let val = noise.sample(x as f32, y as f32);
                assert!(val >= 0.0 && val <= 1.0);
            }
        }
    }

    #[test]
    fn test_value_noise_deterministic() {
        let noise1 = ValueNoise::new(42);
        let noise2 = ValueNoise::new(42);

        for x in 0..10 {
            for y in 0..10 {
                assert!((noise1.sample(x as f32, y as f32) - noise2.sample(x as f32, y as f32))
                    .abs()
                    < f32::EPSILON);
            }
        }
    }

    #[test]
    fn test_value_noise_different_seeds() {
        let noise1 = ValueNoise::new(1);
        let noise2 = ValueNoise::new(2);

        // Should produce different values
        let val1 = noise1.sample(5.0, 5.0);
        let val2 = noise2.sample(5.0, 5.0);
        assert!((val1 - val2).abs() > f32::EPSILON);
    }

    // Dungeon tests
    #[test]
    fn test_dungeon_tile_walkable() {
        assert!(!DungeonTile::Wall.is_walkable());
        assert!(DungeonTile::Floor.is_walkable());
        assert!(DungeonTile::Door.is_walkable());
        assert!(DungeonTile::Corridor.is_walkable());
    }

    #[test]
    fn test_room_center() {
        let room = Room::new(0, 0, 10, 10);
        assert_eq!(room.center(), (5, 5));
    }

    #[test]
    fn test_room_intersects() {
        let room1 = Room::new(0, 0, 10, 10);
        let room2 = Room::new(5, 5, 10, 10);
        let room3 = Room::new(20, 20, 5, 5);

        assert!(room1.intersects(&room2));
        assert!(!room1.intersects(&room3));
    }

    #[test]
    fn test_dungeon_get_set() {
        let mut dungeon = Dungeon::new(10, 10);
        assert_eq!(dungeon.get(5, 5), Some(DungeonTile::Wall));

        dungeon.set(5, 5, DungeonTile::Floor);
        assert_eq!(dungeon.get(5, 5), Some(DungeonTile::Floor));
    }

    #[test]
    fn test_dungeon_in_bounds() {
        let dungeon = Dungeon::new(10, 10);
        assert!(dungeon.in_bounds(5, 5));
        assert!(!dungeon.in_bounds(-1, 5));
        assert!(!dungeon.in_bounds(10, 5));
    }

    #[test]
    fn test_dungeon_generator() {
        let gen = DungeonGenerator::new(50, 50)
            .with_room_size(4, 8)
            .with_room_count(5);

        let dungeon = gen.generate(42).unwrap();

        assert!(!dungeon.rooms.is_empty());
        assert!(dungeon.rooms.len() <= 5);
    }

    #[test]
    fn test_dungeon_walkable_positions() {
        let gen = DungeonGenerator::new(30, 30).with_room_count(3);
        let dungeon = gen.generate(42).unwrap();

        let walkable = dungeon.walkable_positions();
        assert!(!walkable.is_empty());
    }

    // WFC tests
    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_adjacency_rules() {
        let mut rules = AdjacencyRules::new(3);
        rules.add(0, Direction::Right, 1);
        rules.add(0, Direction::Right, 2);

        let allowed = rules.allowed(0, Direction::Right).unwrap();
        assert!(allowed.contains(&1));
        assert!(allowed.contains(&2));
        assert!(!allowed.contains(&0));
    }

    #[test]
    fn test_wfc_cell_entropy() {
        let tiles = vec![0, 1, 2];
        let cell = WfcCell::new(&tiles);
        assert_eq!(cell.entropy(), 3);
        assert!(!cell.is_collapsed());
    }

    #[test]
    fn test_wfc_simple() {
        // Create a simple 2x2 WFC with 2 tile types
        let mut wfc = Wfc::new(2, 2, 2, 42);

        // Add rules: tile 0 can be next to 0 or 1, tile 1 only next to 0
        for dir in Direction::ALL {
            wfc.rules_mut().add(0, dir, 0);
            wfc.rules_mut().add(0, dir, 1);
            wfc.rules_mut().add(1, dir, 0);
        }

        let result = wfc.collapse();
        assert!(result.is_ok());

        // All cells should be collapsed
        let grid = wfc.result();
        assert_eq!(grid.len(), 4);
        for cell in grid {
            assert!(cell.is_some());
        }
    }
}
