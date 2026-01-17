# Tile-Based Parallel Navmesh Generation

This document describes the tile-based parallelism feature added to the rerecast navmesh generator.

## Overview

The tile-based parallelism feature allows navmesh generation to be performed in parallel across multiple CPU cores by dividing the input geometry into tiles on the XZ-plane. Each tile is processed independently, dramatically improving generation times for large environments.

## Usage

### Enabling the Feature

The tile-based parallelism is controlled by the `parallel` feature flag in `Cargo.toml`:

```toml
[dependencies]
rerecast = { version = "0.2.0", features = ["parallel"] }
```

The `parallel` feature is enabled by default. To disable it:

```toml
[dependencies]
rerecast = { version = "0.2.0", default-features = false, features = ["std", "tracing"] }
```

### Configuring Tiled Generation

To use tile-based generation, configure your `Config` with tiling enabled:

```rust
use rerecast::{ConfigBuilder, TiledNavmeshConfig, TriMesh};

let config = ConfigBuilder {
    agent_radius: 0.6,
    agent_height: 2.0,
    tiling: true,        // Enable tiling
    tile_size: 32,       // Size of each tile in voxels
    aabb: trimesh.compute_aabb().unwrap(),
    ..Default::default()
}
.build();

// Create the tiled configuration
let tiled_config = TiledNavmeshConfig::new(config)
    .expect("Failed to create tiled config");

println!("Will generate {} tiles ({} x {})",
    tiled_config.tile_count(),
    tiled_config.tiles_x,
    tiled_config.tiles_z
);
```

### Generating Tiles

Once you have a `TiledNavmeshConfig`, you can generate all tiles:

```rust
// Generate tiles (parallel if feature enabled, sequential otherwise)
let tiles = tiled_config
    .generate_tiles(&trimesh)
    .expect("Failed to generate tiles");

// Process each generated tile
for tile in &tiles {
    println!("Tile at ({}, {}):", tile.coord.x, tile.coord.z);
    println!("  - {} polygons", tile.poly_mesh.polygon_count());
    println!("  - {} detail vertices", tile.detail_mesh.vertices.len());
}
```

## API Reference

### `TiledNavmeshConfig`

Main configuration struct for tiled navmesh generation.

**Methods:**
- `new(config: Config) -> Result<Self, TiledNavmeshError>` - Creates a new tiled config from a base config
- `tile_count() -> usize` - Returns the total number of tiles
- `tile_coords() -> Iterator<Item = TileCoord>` - Returns an iterator over all tile coordinates
- `tile_aabb(coord: TileCoord) -> Aabb3d` - Calculates the AABB for a specific tile
- `generate_tiles(&self, trimesh: &TriMesh) -> Result<Vec<NavmeshTile>, TiledNavmeshError>` - Generates all tiles

### `TileCoord`

Represents a tile coordinate on the XZ plane.

**Fields:**
- `x: u16` - X coordinate of the tile
- `z: u16` - Z coordinate of the tile

### `NavmeshTile`

Represents a single generated navmesh tile.

**Fields:**
- `coord: TileCoord` - The tile's coordinate
- `poly_mesh: PolygonNavmesh` - The tile's polygon mesh
- `detail_mesh: DetailNavmesh` - The tile's detail mesh

## Performance Considerations

### Tile Size

The `tile_size` parameter in `ConfigBuilder` controls the size of each tile in voxels. Choosing an appropriate tile size is important for performance:

- **Too small**: Overhead from managing many tiles can reduce performance
- **Too large**: Less parallelism benefit, less memory locality
- **Recommended**: 32-64 voxels is a good starting point for most environments

### Border Size

Each tile includes a border area (controlled by `border_size` in the config) to ensure proper connectivity between tiles. The border is automatically calculated based on the agent radius.

### Memory Usage

Each tile is processed independently, which means:
- Peak memory usage is approximately: `single_tile_memory * num_cores`
- This can be significant for large tile sizes or many CPU cores
- Consider your available memory when choosing tile size

## Technical Details

### Tile Boundaries

Tiles are created with overlapping borders to ensure proper mesh connectivity at tile boundaries. The border size is automatically calculated as:

```rust
border_size = walkable_radius + 3
```

### Parallel Processing

When the `parallel` feature is enabled, tiles are processed using Rayon's parallel iterators:

```rust
#[cfg(feature = "parallel")]
pub fn generate_tiles_parallel(&self, trimesh: &TriMesh) 
    -> Result<Vec<NavmeshTile>, TiledNavmeshError> 
{
    self.tile_coords()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&coord| self.generate_tile(coord, trimesh))
        .collect()
}
```

Without the `parallel` feature, tiles are processed sequentially:

```rust
#[cfg(not(feature = "parallel"))]
pub fn generate_tiles_sequential(&self, trimesh: &TriMesh) 
    -> Result<Vec<NavmeshTile>, TiledNavmeshError> 
{
    self.tile_coords()
        .map(|coord| self.generate_tile(coord, trimesh))
        .collect()
}
```

### Tile Processing Pipeline

For each tile, the following steps are performed:

1. Calculate tile AABB (including border)
2. Build heightfield for the tile
3. Rasterize triangles that intersect the tile
4. Build compact heightfield
5. Erode walkable area
6. Mark convex volumes
7. Build distance field
8. Build regions
9. Build contours
10. Build polygon mesh
11. Build detail mesh

## Error Handling

The tiled generation system provides detailed error types:

```rust
pub enum TiledNavmeshError {
    HeightfieldBuild(String),
    Rasterization(String),
    CompactHeightfield(String),
    RegionBuild(String),
    PolygonMesh(String),
    DetailMesh(String),
    TilingNotEnabled,
}
```

Each error includes context about which stage of generation failed, making debugging easier.

## Limitations

1. **No Tile Merging**: This implementation generates independent tiles but does not merge them into a single cohesive mesh. Each tile must be used independently or a custom merging solution must be implemented.

2. **Full Geometry Required**: All tiles receive the full input geometry. For very large meshes, consider implementing spatial partitioning of the input geometry for additional memory savings.

3. **Fixed Tile Grid**: Tiles are arranged in a regular grid. Irregular tile layouts are not supported.

## Future Improvements

Potential enhancements for future versions:

1. **Tile Merging**: Implement a merge step to combine tiles into a single mesh
2. **Geometry Culling**: Only pass relevant geometry to each tile
3. **Incremental Updates**: Update only changed tiles when geometry changes
4. **Streaming**: Generate and stream tiles on-demand
5. **Adaptive Tile Sizes**: Automatically adjust tile size based on geometry complexity
