//! Example demonstrating tile-based parallel navmesh generation.
//!
//! This example shows how to use the tiled navmesh generator to process large
//! environments in parallel for improved performance.

use glam::Vec3;
use rerecast::{
    AreaType, ConfigBuilder, HeightfieldBuilder, TiledNavmeshConfig, TriMesh,
};

fn main() {
    // Create a simple ground plane for demonstration
    let mut trimesh = create_test_geometry();
    
    // Configure the navmesh with tiling enabled
    let config = ConfigBuilder {
        agent_radius: 0.6,
        agent_height: 2.0,
        tiling: true,
        tile_size: 32,
        aabb: trimesh.compute_aabb().unwrap(),
        ..Default::default()
    }
    .build();
    
    // Mark walkable triangles
    trimesh.mark_walkable_triangles(config.walkable_slope_angle);
    
    // Create tiled navmesh config
    let tiled_config = TiledNavmeshConfig::new(config).expect("Failed to create tiled config");
    
    println!("Generating navmesh with {} tiles ({} x {})",
        tiled_config.tile_count(),
        tiled_config.tiles_x,
        tiled_config.tiles_z
    );
    
    // Generate all tiles in parallel
    let start = std::time::Instant::now();
    let tiles = tiled_config
        .generate_tiles(&trimesh)
        .expect("Failed to generate tiles");
    let duration = start.elapsed();
    
    println!("Generated {} tiles in {:?}", tiles.len(), duration);
    
    // Print some statistics about the generated tiles
    for (i, tile) in tiles.iter().enumerate() {
        println!(
            "Tile {} at ({}, {}): {} polygons, {} detail vertices",
            i,
            tile.coord.x,
            tile.coord.z,
            tile.poly_mesh.polygon_count(),
            tile.detail_mesh.vertices.len()
        );
    }
}

fn create_test_geometry() -> TriMesh {
    // Create a large flat ground plane (100x100 units)
    let size = 100.0;
    let subdivisions = 20;
    let step = size / subdivisions as f32;
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Generate a grid of vertices
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            vertices.push(Vec3::new(
                x as f32 * step,
                0.0,
                z as f32 * step,
            ));
        }
    }
    
    // Generate triangles
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i0 = z * (subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + (subdivisions + 1);
            let i3 = i2 + 1;
            
            // First triangle
            indices.push([i0 as u32, i2 as u32, i1 as u32]);
            // Second triangle
            indices.push([i1 as u32, i2 as u32, i3 as u32]);
        }
    }
    
    TriMesh {
        vertices,
        indices,
        area_types: vec![AreaType::DEFAULT_WALKABLE; indices.len()],
    }
}
