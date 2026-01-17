//! Benchmark comparing tiled parallel generation vs single-mesh generation for large maps.
//!
//! This benchmark creates progressively larger maps and compares:
//! 1. Single-mesh generation (traditional approach)
//! 2. Tiled generation with sequential processing (no parallelism)
//! 3. Tiled generation with parallel processing (using rayon)
//!
//! Run with: cargo bench --features parallel

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{UVec3, Vec3A};
use rerecast::{
    AreaType, Config, ConfigBuilder, DetailNavmesh, HeightfieldBuilder, TiledNavmeshConfig,
    TriMesh,
};
use std::time::Duration;

/// Generate a large flat terrain with some obstacles
fn generate_large_terrain(size: f32, subdivisions: u32) -> TriMesh {
    let step = size / subdivisions as f32;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate a grid of vertices
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let height = if x % 10 == 0 && z % 10 == 0 {
                // Create some elevated areas as obstacles
                2.0
            } else if (x + z) % 20 < 2 {
                // Create some walls
                5.0
            } else {
                // Flat ground with slight noise
                0.1 * ((x as f32 * 0.1).sin() + (z as f32 * 0.1).cos())
            };

            vertices.push(Vec3A::new(x as f32 * step, height, z as f32 * step));
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
            indices.push(UVec3::new(i0, i2, i1));
            // Second triangle
            indices.push(UVec3::new(i1, i2, i3));
        }
    }

    let area_types = vec![AreaType::DEFAULT_WALKABLE; indices.len()];
    
    TriMesh {
        vertices,
        indices,
        area_types,
    }
}

/// Generate navmesh using traditional single-mesh approach
fn generate_single_mesh(config: &Config, trimesh: &TriMesh) -> (rerecast::PolygonNavmesh, DetailNavmesh) {
    let mut heightfield = HeightfieldBuilder {
        aabb: config.aabb,
        cell_size: config.cell_size,
        cell_height: config.cell_height,
    }
    .build()
    .unwrap();

    heightfield
        .populate_from_trimesh(
            trimesh.clone(),
            config.walkable_height,
            config.walkable_climb,
        )
        .unwrap();

    let mut compact_heightfield = heightfield
        .into_compact(config.walkable_height, config.walkable_climb)
        .unwrap();

    compact_heightfield.erode_walkable_area(config.walkable_radius);

    for volume in &config.area_volumes {
        compact_heightfield.mark_convex_poly_area(volume);
    }

    compact_heightfield.build_distance_field();

    compact_heightfield
        .build_regions(
            config.border_size,
            config.min_region_area,
            config.merge_region_area,
        )
        .unwrap();

    let contours = compact_heightfield.build_contours(
        config.max_simplification_error,
        config.max_edge_len,
        config.contour_flags,
    );

    let poly_mesh = contours
        .into_polygon_mesh(config.max_vertices_per_polygon)
        .unwrap();

    let detail_mesh = DetailNavmesh::new(
        &poly_mesh,
        &compact_heightfield,
        config.detail_sample_dist,
        config.detail_sample_max_error,
    )
    .unwrap();

    (poly_mesh, detail_mesh)
}

/// Benchmark single-mesh generation
fn bench_single_mesh(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_mesh");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    // Test different map sizes
    let sizes = vec![
        ("100x100", 100.0, 50),
        ("200x200", 200.0, 100),
        ("500x500", 500.0, 200),
        ("1000x1000", 1000.0, 400),
    ];

    for (name, size, subdivisions) in sizes {
        let mut trimesh = generate_large_terrain(size, subdivisions);
        let aabb = trimesh.compute_aabb().unwrap();

        let config = ConfigBuilder {
            agent_radius: 0.6,
            agent_height: 2.0,
            tiling: false,
            aabb,
            ..Default::default()
        }
        .build();

        trimesh.mark_walkable_triangles(config.walkable_slope_angle);

        group.throughput(Throughput::Elements(trimesh.indices.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _| {
            b.iter(|| {
                let (poly_mesh, detail_mesh) =
                    generate_single_mesh(black_box(&config), black_box(&trimesh));
                black_box((poly_mesh, detail_mesh))
            });
        });
    }

    group.finish();
}

/// Benchmark tiled generation (sequential)
fn bench_tiled_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiled_sequential");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    let sizes = vec![
        ("100x100_t16", 100.0, 50, 16),
        ("200x200_t16", 200.0, 100, 16),
        ("500x500_t32", 500.0, 200, 32),
        ("1000x1000_t32", 1000.0, 400, 32),
    ];

    for (name, size, subdivisions, tile_size) in sizes {
        let mut trimesh = generate_large_terrain(size, subdivisions);
        let aabb = trimesh.compute_aabb().unwrap();

        let config = ConfigBuilder {
            agent_radius: 0.6,
            agent_height: 2.0,
            tiling: true,
            tile_size,
            aabb,
            ..Default::default()
        }
        .build();

        trimesh.mark_walkable_triangles(config.walkable_slope_angle);

        let tiled_config = TiledNavmeshConfig::new(config).unwrap();

        group.throughput(Throughput::Elements(trimesh.indices.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _| {
            b.iter(|| {
                let tiles =
                    tiled_config.generate_tiles_sequential(black_box(&trimesh)).unwrap();
                black_box(tiles)
            });
        });
    }

    group.finish();
}

/// Benchmark tiled generation (parallel)
#[cfg(feature = "parallel")]
fn bench_tiled_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiled_parallel");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    let sizes = vec![
        ("100x100_t16", 100.0, 50, 16),
        ("200x200_t16", 200.0, 100, 16),
        ("500x500_t32", 500.0, 200, 32),
        ("1000x1000_t32", 1000.0, 400, 32),
        ("2000x2000_t48", 2000.0, 600, 48),
    ];

    for (name, size, subdivisions, tile_size) in sizes {
        let mut trimesh = generate_large_terrain(size, subdivisions);
        let aabb = trimesh.compute_aabb().unwrap();

        let config = ConfigBuilder {
            agent_radius: 0.6,
            agent_height: 2.0,
            tiling: true,
            tile_size,
            aabb,
            ..Default::default()
        }
        .build();

        trimesh.mark_walkable_triangles(config.walkable_slope_angle);

        let tiled_config = TiledNavmeshConfig::new(config).unwrap();

        group.throughput(Throughput::Elements(trimesh.indices.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _| {
            b.iter(|| {
                let tiles = tiled_config.generate_tiles_parallel(black_box(&trimesh)).unwrap();
                black_box(tiles)
            });
        });
    }

    group.finish();
}

/// Comparison benchmark - all approaches on the same map
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_large_map");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    // Use a moderately large map for comparison (smaller for faster benchmarking)
    let size = 500.0;
    let subdivisions = 200;
    let tile_size = 32;

    let mut trimesh = generate_large_terrain(size, subdivisions);
    let aabb = trimesh.compute_aabb().unwrap();

    println!(
        "Generated terrain: {} vertices, {} triangles",
        trimesh.vertices.len(),
        trimesh.indices.len()
    );

    // Single mesh config
    let single_config = ConfigBuilder {
        agent_radius: 0.6,
        agent_height: 2.0,
        tiling: false,
        aabb,
        ..Default::default()
    }
    .build();

    // Tiled config
    let tiled_config_base = ConfigBuilder {
        agent_radius: 0.6,
        agent_height: 2.0,
        tiling: true,
        tile_size,
        aabb,
        ..Default::default()
    }
    .build();

    trimesh.mark_walkable_triangles(single_config.walkable_slope_angle);

    let tiled_config = TiledNavmeshConfig::new(tiled_config_base).unwrap();
    println!(
        "Tiled config: {} tiles ({} x {})",
        tiled_config.tile_count(),
        tiled_config.tiles_x,
        tiled_config.tiles_z
    );

    group.throughput(Throughput::Elements(trimesh.indices.len() as u64));

    // Benchmark single mesh
    group.bench_function("single_mesh", |b| {
        b.iter(|| {
            let (poly_mesh, detail_mesh) =
                generate_single_mesh(black_box(&single_config), black_box(&trimesh));
            black_box((poly_mesh, detail_mesh))
        });
    });

    // Benchmark sequential tiled
    group.bench_function("tiled_sequential", |b| {
        b.iter(|| {
            let tiles = tiled_config
                .generate_tiles_sequential(black_box(&trimesh))
                .unwrap();
            black_box(tiles)
        });
    });

    // Benchmark parallel tiled
    #[cfg(feature = "parallel")]
    group.bench_function("tiled_parallel", |b| {
        b.iter(|| {
            let tiles = tiled_config
                .generate_tiles_parallel(black_box(&trimesh))
                .unwrap();
            black_box(tiles)
        });
    });

    group.finish();
}

/// Scalability benchmark - test how performance scales with map size
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(90));
    group.sample_size(10);

    let configs = vec![
        ("250x250", 250.0, 100),
        ("500x500", 500.0, 200),
        ("750x750", 750.0, 300),
        ("1000x1000", 1000.0, 400),
    ];

    for (name, size, subdivisions) in configs {
        let mut trimesh = generate_large_terrain(size, subdivisions);
        let aabb = trimesh.compute_aabb().unwrap();
        trimesh.mark_walkable_triangles(45.0_f32.to_radians());

        let triangle_count = trimesh.indices.len() as u64;
        group.throughput(Throughput::Elements(triangle_count));

        // Single mesh
        let single_config = ConfigBuilder {
            agent_radius: 0.6,
            agent_height: 2.0,
            tiling: false,
            aabb,
            ..Default::default()
        }
        .build();

        group.bench_with_input(
            BenchmarkId::new("single", name),
            &(&single_config, &trimesh),
            |b, (config, mesh)| {
                b.iter(|| {
                    let result = generate_single_mesh(black_box(config), black_box(mesh));
                    black_box(result)
                });
            },
        );

        // Parallel tiled
        #[cfg(feature = "parallel")]
        {
            let tiled_config_base = ConfigBuilder {
                agent_radius: 0.6,
                agent_height: 2.0,
                tiling: true,
                tile_size: 32,
                aabb,
                ..Default::default()
            }
            .build();

            let tiled_config = TiledNavmeshConfig::new(tiled_config_base).unwrap();

            group.bench_with_input(
                BenchmarkId::new("parallel", name),
                &(&tiled_config, &trimesh),
                |b, (config, mesh)| {
                    b.iter(|| {
                        let tiles = config.generate_tiles_parallel(black_box(mesh)).unwrap();
                        black_box(tiles)
                    });
                },
            );
        }
    }

    group.finish();
}

#[cfg(feature = "parallel")]
criterion_group!(
    benches,
    bench_single_mesh,
    bench_tiled_sequential,
    bench_tiled_parallel,
    bench_comparison,
    bench_scalability
);

#[cfg(not(feature = "parallel"))]
criterion_group!(
    benches,
    bench_single_mesh,
    bench_tiled_sequential,
    bench_comparison
);

criterion_main!(benches);
