/// Spatial partitioning for large-scale space
///
/// For a game with 1000x scale nebulas and planetary distances,
/// we need efficient spatial queries. This module provides:
/// - Sector-based space partitioning
/// - Efficient "nearby entity" queries
/// - LOD (Level of Detail) management

use glam::DVec3;
use std::collections::HashMap;

/// Sector coordinates for spatial partitioning
/// Each sector is a large cubic region of space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl SectorCoord {
    /// Create sector coordinate from world position
    /// sector_size: size of each sector in meters
    pub fn from_position(pos: DVec3, sector_size: f64) -> Self {
        Self {
            x: (pos.x / sector_size).floor() as i64,
            y: (pos.y / sector_size).floor() as i64,
            z: (pos.z / sector_size).floor() as i64,
        }
    }

    /// Get neighboring sectors (3x3x3 cube centered on this sector)
    pub fn neighbors(&self) -> Vec<SectorCoord> {
        let mut neighbors = Vec::with_capacity(27);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    neighbors.push(SectorCoord {
                        x: self.x + dx,
                        y: self.y + dy,
                        z: self.z + dz,
                    });
                }
            }
        }
        neighbors
    }
}

/// Spatial index for efficient entity queries
pub struct SpatialIndex {
    /// Maps sector coordinates to list of entity IDs in that sector
    sectors: HashMap<SectorCoord, Vec<u64>>,

    /// Size of each sector in meters
    /// Larger sectors = fewer sectors but less precise queries
    /// Recommend: 10,000 meters (10km) for ship combat
    ///            1,000,000 meters (1000km) for planetary scale
    sector_size: f64,
}

impl SpatialIndex {
    pub fn new(sector_size: f64) -> Self {
        Self {
            sectors: HashMap::new(),
            sector_size,
        }
    }

    /// Clear all entities from the index
    pub fn clear(&mut self) {
        self.sectors.clear();
    }

    /// Insert an entity at a position
    pub fn insert(&mut self, entity_id: u64, position: DVec3) {
        let sector = SectorCoord::from_position(position, self.sector_size);
        self.sectors.entry(sector).or_default().push(entity_id);
    }

    /// Query entities near a position
    /// Returns all entities in the same sector and neighboring sectors
    pub fn query_nearby(&self, position: DVec3) -> Vec<u64> {
        let sector = SectorCoord::from_position(position, self.sector_size);
        let mut result = Vec::new();

        for neighbor_sector in sector.neighbors() {
            if let Some(entities) = self.sectors.get(&neighbor_sector) {
                result.extend_from_slice(entities);
            }
        }

        result
    }

    /// Query entities within a radius
    /// More expensive but more accurate than sector query
    pub fn query_radius(
        &self,
        position: DVec3,
        radius: f64,
        entity_positions: &HashMap<u64, DVec3>,
    ) -> Vec<u64> {
        let nearby = self.query_nearby(position);
        let radius_sq = radius * radius;

        nearby
            .into_iter()
            .filter(|&entity_id| {
                if let Some(&entity_pos) = entity_positions.get(&entity_id) {
                    let dist_sq = position.distance_squared(entity_pos);
                    dist_sq <= radius_sq
                } else {
                    false
                }
            })
            .collect()
    }
}

/// LOD (Level of Detail) manager
/// Determines rendering detail based on distance from camera
pub struct LodManager {
    /// Distance thresholds for LOD levels
    /// [high_detail, medium_detail, low_detail, culled]
    pub thresholds: [f64; 4],
}

impl LodManager {
    pub fn new() -> Self {
        Self {
            // Example thresholds (in meters):
            // 0-1000m: high detail (full mesh + physics)
            // 1000-10000m: medium detail (simplified mesh + physics)
            // 10000-100000m: low detail (billboard/sprite + no physics)
            // >100000m: culled (not rendered)
            thresholds: [1000.0, 10_000.0, 100_000.0, 1_000_000.0],
        }
    }

    /// Get LOD level for a given distance
    pub fn get_lod_level(&self, distance: f64) -> LodLevel {
        if distance < self.thresholds[0] {
            LodLevel::High
        } else if distance < self.thresholds[1] {
            LodLevel::Medium
        } else if distance < self.thresholds[2] {
            LodLevel::Low
        } else if distance < self.thresholds[3] {
            LodLevel::Billboard
        } else {
            LodLevel::Culled
        }
    }
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodLevel {
    High,       // Full detail mesh + full physics
    Medium,     // Simplified mesh + simplified physics
    Low,        // Very simple mesh + no physics
    Billboard,  // Single sprite/billboard
    Culled,     // Not rendered at all
}
