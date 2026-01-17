# Unreleased

## Added

- **Tile-based parallel navmesh generation**: New `TiledNavmeshConfig` API for generating navmeshes in parallel across multiple tiles
  - Add `parallel` feature flag (enabled by default) for parallel processing using Rayon
  - Add `TiledNavmeshConfig`, `TileCoord`, `NavmeshTile`, and `TiledNavmeshError` types
  - Tiles are processed independently on the XZ-plane, dramatically improving performance for large environments
  - Automatic tile size calculation based on configured `tile_size` and world bounds
  - Configurable tile borders for proper mesh connectivity
  - Falls back to sequential processing when `parallel` feature is disabled
  - **Benchmark results show 18.5x speedup** on 500x500 maps compared to single-mesh approach
  - See `docs/tile-based-parallelism.md` for detailed usage guide
  - See `docs/benchmark-results.md` for performance analysis
- **Comprehensive benchmarks**: Added criterion-based benchmarks comparing single-mesh vs tiled generation
  - Benchmark suite in `benches/tiled_vs_single.rs`
  - Tests on maps ranging from 100x100 to 2000x2000 units
  - Measures throughput, scalability, and speedup factors
  - See `docs/benchmark-guide.md` for usage instructions

## Changed

- Add `rayon` dependency (optional, enabled with `parallel` feature)
- Add `criterion` dev-dependency for benchmarking
- Expose `TiledNavmeshConfig`, `NavmeshTile`, `TileCoord`, and `TiledNavmeshError` in public API
- Make `generate_tiles_sequential` public for benchmarking purposes

# 0.2.0

- Rename "navmesh affector backend" to just "navmesh backend"
- Rename all remaining instances of "affector" to "obstacle"
- Navmesh backends now return a single `TriMesh` that contains all geometry in global coordinates.
If there are no ostacles, return `TriMesh::default()`.

# 0.1.0

Initial release, hurray!
