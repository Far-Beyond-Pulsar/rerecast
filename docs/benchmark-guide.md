# Navmesh Generation Benchmark Guide

This guide explains how to run and interpret the benchmarks comparing single-mesh vs tile-based parallel generation.

## Running the Benchmarks

### Full Benchmark Suite

Run all benchmarks (this will take a while - expect 20-30 minutes):

```bash
cd crates/rerecast
cargo bench --features parallel
```

### Individual Benchmarks

Run specific benchmark groups:

```bash
# Single-mesh approach only
cargo bench --features parallel -- single_mesh

# Tiled sequential approach only
cargo bench --features parallel -- tiled_sequential

# Tiled parallel approach only
cargo bench --features parallel -- tiled_parallel

# Direct comparison on 1000x1000 map
cargo bench --features parallel -- comparison_large_map

# Scalability analysis
cargo bench --features parallel -- scalability
```

### Quick Benchmark (Faster)

For a quicker test with fewer samples:

```bash
cargo bench --features parallel -- --sample-size 5
```

## Benchmark Descriptions

### 1. `single_mesh`
Tests the traditional single-mesh generation approach on maps of varying sizes:
- 100x100 units (50 subdivisions)
- 200x200 units (100 subdivisions)
- 500x500 units (200 subdivisions)
- 1000x1000 units (400 subdivisions)

### 2. `tiled_sequential`
Tests tile-based generation without parallelism (sequential processing):
- Same map sizes as single_mesh
- Uses tile sizes of 16-32 voxels
- Shows overhead of tiling without parallel benefit

### 3. `tiled_parallel`
Tests tile-based generation with parallelism enabled:
- Same map sizes as single_mesh, plus:
- 2000x2000 units (600 subdivisions) - very large map
- Uses tile sizes of 16-48 voxels
- Shows performance improvement from parallel processing

### 4. `comparison_large_map`
Direct comparison of all three approaches on the same 1000x1000 map:
- Single mesh generation
- Tiled sequential generation
- Tiled parallel generation

### 5. `scalability`
Tests how each approach scales with increasing map size:
- Measures performance on 250x250, 500x500, 750x750, and 1000x1000 maps
- Compares single-mesh vs parallel tiled approaches
- Shows throughput (triangles per second)

## Understanding Results

### Output Location

Results are saved to `crates/rerecast/target/criterion/`:
- HTML reports: Open `index.html` in a browser
- Raw data: CSV files for each benchmark

### Key Metrics

**Time**: Lower is better
- Single-mesh: Linear growth with map size
- Tiled parallel: Sub-linear growth (benefits from parallelism)

**Throughput**: Higher is better
- Measured in triangles/second
- Shows how efficiently each approach processes geometry

**Speedup Factor**:
```
Speedup = (Single-mesh time) / (Tiled parallel time)
```
- Values > 1 indicate tiled parallel is faster
- Values near CPU core count indicate good parallelization

### Expected Results

On a typical multi-core system (e.g., 8 cores):

| Map Size | Single Mesh | Tiled Sequential | Tiled Parallel | Speedup |
|----------|-------------|------------------|----------------|---------|
| 100x100  | ~100ms      | ~120ms           | ~80ms          | 1.25x   |
| 500x500  | ~2s         | ~2.2s            | ~400ms         | 5x      |
| 1000x1000| ~15s        | ~16s             | ~2.5s          | 6x      |
| 2000x2000| ~90s        | ~95s             | ~12s           | 7.5x    |

*Note: Actual times depend on CPU, memory, and system load*

### Interpreting Scalability

Good scalability characteristics:
- Tiled parallel shows sub-linear time growth
- Speedup increases with map size
- Throughput remains relatively constant or increases slightly

Poor scalability would show:
- Linear or super-linear time growth
- Decreasing speedup with larger maps
- Decreasing throughput

## Benchmark Parameters

### Map Generation

Each test map includes:
- Flat ground plane with slight noise
- Elevated obstacles (every 10th grid point)
- Wall segments (creates corridors)
- All geometry marked as walkable

### Navmesh Configuration

Standard configuration for all tests:
- Agent radius: 0.6 units
- Agent height: 2.0 units
- Cell size: ~0.3 units (agent_radius / 2)
- Cell height: ~0.15 units (agent_radius / 4)

### Tile Configuration

For tiled approaches:
- Small maps (100-200): 16 voxel tiles
- Medium maps (500): 32 voxel tiles
- Large maps (1000-2000): 32-48 voxel tiles

## Performance Tips

### For Benchmarking

1. **Close other applications** to reduce system noise
2. **Disable power saving** modes for consistent results
3. **Run multiple times** and average results
4. **Check CPU temperature** - thermal throttling affects results

### For Production Use

1. **Choose appropriate tile size**:
   - Smaller tiles (16-24): Better for very large maps, more parallelism
   - Larger tiles (32-48): Better cache locality, less overhead
   - Rule of thumb: `tile_size = sqrt(map_size) / 4`

2. **Consider memory**:
   - Peak memory ≈ `tile_memory × CPU_cores`
   - Monitor memory usage during generation

3. **Profile your specific maps**:
   - Benchmark with representative geometry
   - Adjust tile size based on complexity
   - Test with different thread counts

## Troubleshooting

### Benchmark Fails to Compile

Ensure you have the `parallel` feature enabled:
```bash
cargo bench --features parallel
```

### Benchmark Takes Too Long

Reduce sample size:
```bash
cargo bench --features parallel -- --sample-size 5
```

Or run specific benchmarks:
```bash
cargo bench --features parallel -- comparison_large_map
```

### Out of Memory

If benchmarks crash due to memory:
1. Reduce map sizes in `benches/tiled_vs_single.rs`
2. Increase tile sizes to reduce tile count
3. Close other applications

### Inconsistent Results

If results vary significantly between runs:
1. Close background applications
2. Ensure CPU isn't thermal throttling
3. Increase sample size for more reliable statistics
4. Check system monitoring tools for interference

## Customizing Benchmarks

To modify the benchmarks:

1. Edit `crates/rerecast/benches/tiled_vs_single.rs`
2. Adjust map sizes in the `sizes` vectors
3. Modify terrain complexity in `generate_large_terrain()`
4. Change configuration parameters
5. Rebuild with `cargo bench --no-run --features parallel`

## Analyzing Results

### Using Criterion's HTML Reports

1. Open `target/criterion/report/index.html`
2. Navigate to specific benchmarks
3. View violin plots for distribution
4. Check regression detection
5. Compare with previous runs

### Exporting Data

Raw data is available in CSV format:
```bash
target/criterion/<benchmark_name>/base/raw.csv
```

Import into spreadsheet software for custom analysis.

## Contributing Benchmark Improvements

When contributing benchmark modifications:

1. Ensure benchmarks remain reproducible
2. Document any new parameters
3. Include expected performance characteristics
4. Test on multiple systems if possible
5. Update this guide with new information
