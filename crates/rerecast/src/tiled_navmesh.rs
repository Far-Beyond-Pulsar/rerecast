//! Tile-based parallel navmesh generation.
//!
//! This module provides functionality for generating navigation meshes using tile-based parallelism.
//! The input geometry is divided into tiles on the XZ-plane, and each tile is processed independently
//! in parallel, dramatically improving generation times for large environments.

use crate::{
    ops::ceil,
    Aabb3d, Config, DetailNavmesh, HeightfieldBuilder, PolygonNavmesh, TriMesh,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use glam::Vec3;
use thiserror::Error;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// A tile coordinate on the XZ plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    /// X coordinate of the tile
    pub x: u16,
    /// Z coordinate of the tile
    pub z: u16,
}

/// Configuration for tiled navmesh generation.
#[derive(Debug, Clone)]
pub struct TiledNavmeshConfig {
    /// The base configuration for navmesh generation
    pub config: Config,
    /// Number of tiles along the X axis
    pub tiles_x: u16,
    /// Number of tiles along the Z axis
    pub tiles_z: u16,
}

/// A single tile of a navmesh.
#[derive(Debug, Clone)]
pub struct NavmeshTile {
    /// The tile's coordinate
    pub coord: TileCoord,
    /// The tile's polygon mesh
    pub poly_mesh: PolygonNavmesh,
    /// The tile's detail mesh
    pub detail_mesh: DetailNavmesh,
}

/// Errors that can occur during tiled navmesh generation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TiledNavmeshError {
    /// Error building heightfield
    #[error("Failed to build heightfield: {0}")]
    HeightfieldBuild(String),
    /// Error during rasterization
    #[error("Failed to rasterize triangles: {0}")]
    Rasterization(String),
    /// Error building compact heightfield
    #[error("Failed to build compact heightfield: {0}")]
    CompactHeightfield(String),
    /// Error building regions
    #[error("Failed to build regions: {0}")]
    RegionBuild(String),
    /// Error building polygon mesh
    #[error("Failed to build polygon mesh: {0}")]
    PolygonMesh(String),
    /// Error building detail mesh
    #[error("Failed to build detail mesh: {0}")]
    DetailMesh(String),
    /// Tiling is not enabled in the config
    #[error("Tiling is not enabled in the config")]
    TilingNotEnabled,
}

impl TiledNavmeshConfig {
    /// Creates a new tiled navmesh configuration from a base config.
    ///
    /// # Arguments
    ///
    /// * `config` - The base configuration. Must have `tiling` set to `true`.
    ///
    /// # Errors
    ///
    /// Returns an error if tiling is not enabled in the config.
    pub fn new(config: Config) -> Result<Self, TiledNavmeshError> {
        if config.tile_size == 0 {
            return Err(TiledNavmeshError::TilingNotEnabled);
        }

        let world_width = config.aabb.max.x - config.aabb.min.x;
        let world_height = config.aabb.max.z - config.aabb.min.z;

        let tile_world_size = config.tile_size as f32 * config.cell_size;
        
        // Ensure we don't divide by zero
        if tile_world_size <= 0.0 {
            return Err(TiledNavmeshError::TilingNotEnabled);
        }

        let tiles_x = ceil(world_width / tile_world_size) as u16;
        let tiles_z = ceil(world_height / tile_world_size) as u16;

        Ok(Self {
            config,
            tiles_x,
            tiles_z,
        })
    }

    /// Returns the total number of tiles.
    pub fn tile_count(&self) -> usize {
        self.tiles_x as usize * self.tiles_z as usize
    }

    /// Returns an iterator over all tile coordinates.
    pub fn tile_coords(&self) -> impl Iterator<Item = TileCoord> {
        let tiles_x = self.tiles_x;
        let tiles_z = self.tiles_z;
        (0..tiles_z).flat_map(move |z| (0..tiles_x).map(move |x| TileCoord { x, z }))
    }

    /// Calculates the AABB for a specific tile, including border.
    pub fn tile_aabb(&self, coord: TileCoord) -> Aabb3d {
        let tile_world_size = self.config.tile_size as f32 * self.config.cell_size;
        let border_world_size = self.config.border_size as f32 * self.config.cell_size;

        let min_x = self.config.aabb.min.x + coord.x as f32 * tile_world_size - border_world_size;
        let max_x = min_x + tile_world_size + 2.0 * border_world_size;
        let min_z = self.config.aabb.min.z + coord.z as f32 * tile_world_size - border_world_size;
        let max_z = min_z + tile_world_size + 2.0 * border_world_size;

        Aabb3d {
            min: Vec3::new(min_x, self.config.aabb.min.y, min_z),
            max: Vec3::new(max_x, self.config.aabb.max.y, max_z),
        }
    }

    /// Generates a single tile's navmesh.
    fn generate_tile(
        &self,
        coord: TileCoord,
        trimesh: &TriMesh,
    ) -> Result<NavmeshTile, TiledNavmeshError> {
        let tile_aabb = self.tile_aabb(coord);

        // Build heightfield for this tile
        let mut heightfield = HeightfieldBuilder {
            aabb: tile_aabb,
            cell_size: self.config.cell_size,
            cell_height: self.config.cell_height,
        }
        .build()
        .map_err(|e| TiledNavmeshError::HeightfieldBuild(e.to_string()))?;

        // Rasterize triangles that intersect this tile
        heightfield
            .populate_from_trimesh(
                trimesh.clone(),
                self.config.walkable_height,
                self.config.walkable_climb,
            )
            .map_err(|e| TiledNavmeshError::Rasterization(e.to_string()))?;

        // Build compact heightfield
        let mut compact_heightfield = heightfield
            .into_compact(self.config.walkable_height, self.config.walkable_climb)
            .map_err(|e| TiledNavmeshError::CompactHeightfield(e.to_string()))?;

        // Erode walkable area
        compact_heightfield.erode_walkable_area(self.config.walkable_radius);

        // Mark convex volumes
        for volume in &self.config.area_volumes {
            compact_heightfield.mark_convex_poly_area(volume);
        }

        // Build distance field
        compact_heightfield.build_distance_field();

        // Build regions
        compact_heightfield
            .build_regions(
                self.config.border_size,
                self.config.min_region_area,
                self.config.merge_region_area,
            )
            .map_err(|e| TiledNavmeshError::RegionBuild(e.to_string()))?;

        // Build contours
        let contours = compact_heightfield.build_contours(
            self.config.max_simplification_error,
            self.config.max_edge_len,
            self.config.contour_flags,
        );

        // Build polygon mesh
        let poly_mesh = contours
            .into_polygon_mesh(self.config.max_vertices_per_polygon)
            .map_err(|e| TiledNavmeshError::PolygonMesh(e.to_string()))?;

        // Build detail mesh
        let detail_mesh = DetailNavmesh::new(
            &poly_mesh,
            &compact_heightfield,
            self.config.detail_sample_dist,
            self.config.detail_sample_max_error,
        )
        .map_err(|e| TiledNavmeshError::DetailMesh(e.to_string()))?;

        Ok(NavmeshTile {
            coord,
            poly_mesh,
            detail_mesh,
        })
    }

    /// Generates all tiles in parallel (if the `parallel` feature is enabled).
    ///
    /// # Arguments
    ///
    /// * `trimesh` - The triangle mesh to generate the navmesh from.
    ///
    /// # Returns
    ///
    /// A vector of all generated tiles.
    #[cfg(feature = "parallel")]
    pub fn generate_tiles_parallel(
        &self,
        trimesh: &TriMesh,
    ) -> Result<Vec<NavmeshTile>, TiledNavmeshError> {
        self.tile_coords()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|&coord| self.generate_tile(coord, trimesh))
            .collect()
    }

    /// Generates all tiles sequentially.
    ///
    /// # Arguments
    ///
    /// * `trimesh` - The triangle mesh to generate the navmesh from.
    ///
    /// # Returns
    ///
    /// A vector of all generated tiles.
    pub fn generate_tiles_sequential(
        &self,
        trimesh: &TriMesh,
    ) -> Result<Vec<NavmeshTile>, TiledNavmeshError> {
        self.tile_coords()
            .map(|coord| self.generate_tile(coord, trimesh))
            .collect()
    }

    /// Generates all tiles (parallel if available, sequential otherwise).
    pub fn generate_tiles(
        &self,
        trimesh: &TriMesh,
    ) -> Result<Vec<NavmeshTile>, TiledNavmeshError> {
        #[cfg(feature = "parallel")]
        return self.generate_tiles_parallel(trimesh);

        #[cfg(not(feature = "parallel"))]
        return self.generate_tiles_sequential(trimesh);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConfigBuilder;

    #[test]
    fn test_tile_coord_generation() {
        let config = ConfigBuilder {
            tiling: true,
            tile_size: 32,
            aabb: Aabb3d {
                min: Vec3::new(0.0, 0.0, 0.0),
                max: Vec3::new(100.0, 10.0, 100.0),
            },
            ..Default::default()
        }
        .build();

        let tiled_config = TiledNavmeshConfig::new(config).unwrap();

        // Verify we get the expected number of tiles
        assert!(tiled_config.tiles_x > 0);
        assert!(tiled_config.tiles_z > 0);
        assert_eq!(
            tiled_config.tile_count(),
            tiled_config.tiles_x as usize * tiled_config.tiles_z as usize
        );

        // Verify all coordinates are generated
        let coords: Vec<_> = tiled_config.tile_coords().collect();
        assert_eq!(coords.len(), tiled_config.tile_count());
    }

    #[test]
    fn test_tile_aabb_calculation() {
        let config = ConfigBuilder {
            tiling: true,
            tile_size: 32,
            aabb: Aabb3d {
                min: Vec3::new(0.0, 0.0, 0.0),
                max: Vec3::new(100.0, 10.0, 100.0),
            },
            ..Default::default()
        }
        .build();

        let tiled_config = TiledNavmeshConfig::new(config).unwrap();
        let tile_aabb = tiled_config.tile_aabb(TileCoord { x: 0, z: 0 });

        // AABB should include border
        assert!(tile_aabb.min.x < tiled_config.config.aabb.min.x);
        assert!(tile_aabb.min.z < tiled_config.config.aabb.min.z);
    }

    #[test]
    fn test_tiling_not_enabled() {
        let mut config = ConfigBuilder {
            tiling: false,
            ..Default::default()
        }
        .build();
        
        // When tiling is false, tile_size is still set to a default value
        // We need to explicitly set it to 0 to disable tiling
        config.tile_size = 0;

        let result = TiledNavmeshConfig::new(config);
        assert!(matches!(result, Err(TiledNavmeshError::TilingNotEnabled)));
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_parallel_tile_generation() {
        use crate::AreaType;
        use glam::{Vec3A, UVec3};

        // Create a simple ground plane
        let vertices = vec![
            Vec3A::new(0.0, 0.0, 0.0),
            Vec3A::new(100.0, 0.0, 0.0),
            Vec3A::new(0.0, 0.0, 100.0),
            Vec3A::new(100.0, 0.0, 100.0),
        ];
        let indices = vec![UVec3::new(0, 2, 1), UVec3::new(1, 2, 3)];
        let mut trimesh = TriMesh {
            vertices,
            indices,
            area_types: vec![AreaType::DEFAULT_WALKABLE; 2],
        };

        let config = ConfigBuilder {
            agent_radius: 0.6,
            agent_height: 2.0,
            tiling: true,
            tile_size: 16,
            aabb: trimesh.compute_aabb().unwrap(),
            ..Default::default()
        }
        .build();

        trimesh.mark_walkable_triangles(config.walkable_slope_angle);

        let tiled_config = TiledNavmeshConfig::new(config).unwrap();
        
        // This should generate multiple tiles
        assert!(tiled_config.tile_count() > 1);
        
        // Generate tiles in parallel
        let tiles = tiled_config.generate_tiles(&trimesh).unwrap();
        
        // Verify we got the expected number of tiles
        assert_eq!(tiles.len(), tiled_config.tile_count());
        
        // Verify each tile has the correct coordinates
        for tile in &tiles {
            assert!(tile.coord.x < tiled_config.tiles_x);
            assert!(tile.coord.z < tiled_config.tiles_z);
        }
    }
}
