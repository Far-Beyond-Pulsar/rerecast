use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Estimate {
    point_estimate: f64,
}

#[derive(Debug, Deserialize)]
struct BenchmarkData {
    mean: Estimate,
}

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    time_s: f64,
    throughput: f64,
}

fn run_benchmarks() -> Result<()> {
    println!("🔬 Running benchmarks...");
    println!("   This will take a few minutes...\n");
    
    let status = Command::new("cargo")
        .args(&[
            "bench",
            "--features", "parallel",
            "--bench", "tiled_vs_single",
            "--",
            "comparison_large_map"
        ])
        .current_dir("../rerecast")
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Benchmark failed to run");
    }
    
    println!("\n✅ Benchmarks complete!\n");
    Ok(())
}

fn find_criterion_dir() -> Result<PathBuf> {
    let locations = vec![
        PathBuf::from("../../target/criterion"),
        PathBuf::from("../rerecast/target/criterion"),
        PathBuf::from("target/criterion"),
    ];

    for loc in locations {
        if loc.exists() {
            return Ok(loc);
        }
    }

    anyhow::bail!("Could not find Criterion results directory")
}

fn parse_benchmark_results(criterion_dir: &Path) -> Result<Vec<BenchmarkResult>> {
    let group_dir = criterion_dir.join("comparison_large_map");
    if !group_dir.exists() {
        anyhow::bail!("comparison_large_map group not found");
    }

    let mut results = Vec::new();
    let triangles = 80_000.0;

    for entry in fs::read_dir(&group_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let estimate_file = path.join("base/estimates.json");

            if estimate_file.exists() {
                let data = fs::read_to_string(&estimate_file)?;
                let bench_data: BenchmarkData = serde_json::from_str(&data)?;

                let time_s = bench_data.mean.point_estimate / 1_000_000_000.0;
                let throughput = (triangles / time_s) / 1000.0; // K tri/s

                results.push(BenchmarkResult {
                    name,
                    time_s,
                    throughput,
                });
            }
        }
    }

    Ok(results)
}

fn generate_html(results: &[BenchmarkResult]) -> String {
    // Sort results in the correct order
    let mut sorted = Vec::new();
    let order = ["single_mesh", "tiled_sequential", "tiled_parallel"];
    let labels = [
        ("single_mesh", "Single Mesh"),
        ("tiled_sequential", "Tiled Sequential"),
        ("tiled_parallel", "Tiled Parallel"),
    ];

    for (key, label) in &labels {
        if let Some(result) = results.iter().find(|r| r.name == *key) {
            sorted.push((label, result));
        }
    }

    if sorted.is_empty() {
        return String::from("No benchmark results found!");
    }

    let baseline_time = sorted[0].1.time_s;
    let speedups: Vec<f64> = sorted
        .iter()
        .map(|(_, r)| baseline_time / r.time_s)
        .collect();

    let approaches_json = serde_json::to_string(
        &sorted.iter().map(|(label, _)| label).collect::<Vec<_>>()
    ).unwrap();
    
    let times_json = serde_json::to_string(
        &sorted.iter().map(|(_, r)| r.time_s).collect::<Vec<_>>()
    ).unwrap();
    
    let throughputs_json = serde_json::to_string(
        &sorted.iter().map(|(_, r)| r.throughput).collect::<Vec<_>>()
    ).unwrap();
    
    let speedups_json = serde_json::to_string(&speedups).unwrap();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rerecast Benchmark Results</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 2rem;
            line-height: 1.6;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }}
        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 3rem 2rem;
            text-align: center;
        }}
        header h1 {{ font-size: 2.5rem; margin-bottom: 0.5rem; }}
        header p {{ font-size: 1.2rem; opacity: 0.9; }}
        .content {{ padding: 2rem; }}
        section {{ margin-bottom: 3rem; }}
        h2 {{
            color: #667eea;
            font-size: 2rem;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 3px solid #667eea;
        }}
        .highlight-box {{
            background: linear-gradient(135deg, #667eea15 0%, #764ba215 100%);
            border-left: 4px solid #667eea;
            padding: 1.5rem;
            margin: 1.5rem 0;
            border-radius: 8px;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin: 2rem 0;
        }}
        .stat-card {{
            background: white;
            border: 2px solid #e2e8f0;
            border-radius: 12px;
            padding: 1.5rem;
            text-align: center;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
        }}
        .stat-card.winner {{
            background: linear-gradient(135deg, #10b98115 0%, #05966915 100%);
            border-color: #10b981;
        }}
        .stat-card .label {{
            font-size: 0.9rem;
            color: #64748b;
            text-transform: uppercase;
            margin-bottom: 0.5rem;
        }}
        .stat-card .value {{
            font-size: 2rem;
            font-weight: 700;
            color: #667eea;
            margin-bottom: 0.25rem;
        }}
        .stat-card.winner .value {{ color: #059669; }}
        .stat-card .subtext {{ font-size: 0.85rem; color: #64748b; }}
        .chart-container {{
            position: relative;
            height: 400px;
            margin: 2rem 0;
            padding: 1rem;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 2rem 0;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }}
        thead {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }}
        th {{
            padding: 1rem;
            text-align: left;
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.85rem;
        }}
        td {{ padding: 1rem; border-bottom: 1px solid #e2e8f0; }}
        tbody tr:hover {{ background: #f8fafc; }}
        .winner-row {{
            background: #f0fdf415 !important;
            font-weight: 600;
            border-left: 4px solid #10b981;
        }}
        .speedup {{
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-weight: 600;
            font-size: 0.9rem;
        }}
        .speedup-high {{ background: #10b98120; color: #059669; }}
        .speedup-medium {{ background: #f59e0b20; color: #d97706; }}
        .speedup-low {{ background: #64748b20; color: #475569; }}
        footer {{
            background: #1e293b;
            color: white;
            padding: 2rem;
            text-align: center;
        }}
        footer p {{ margin: 0.5rem 0; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🚀 Tile-Based Parallel Navmesh Generation</h1>
            <p>Performance Benchmark Results</p>
            <p style="font-size: 0.9rem; opacity: 0.8; margin-top: 0.5rem;">Auto-generated from Criterion data</p>
        </header>

        <div class="content">
            <section>
                <h2>Executive Summary</h2>
                <div class="highlight-box">
                    <p style="font-size: 1.2rem; font-weight: 600; margin-bottom: 1rem;">
                        The tile-based parallel approach delivers <strong style="color: #059669;">{:.1}× faster</strong> 
                        navmesh generation compared to the traditional single-mesh approach.
                    </p>
                    <p><strong>Real measured data from actual benchmarks.</strong> Tested on a 500×500 map with 80,000 triangles.</p>
                    <p style="margin-top: 0.5rem;">The parallel implementation reduces generation 
                    time from {:.2} seconds to just {:.0} milliseconds.</p>
                </div>
            </section>

            <section>
                <h2>Performance Results</h2>
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="label">{}</div>
                        <div class="value">{:.2}s</div>
                        <div class="subtext">{:.1}K tri/s</div>
                    </div>
                    <div class="stat-card">
                        <div class="label">{}</div>
                        <div class="value">{:.2}s</div>
                        <div class="subtext">{:.1}K tri/s</div>
                    </div>
                    <div class="stat-card winner">
                        <div class="label">{}</div>
                        <div class="value">{:.3}s</div>
                        <div class="subtext">{:.1}K tri/s</div>
                    </div>
                </div>

                <table>
                    <thead>
                        <tr>
                            <th>Approach</th>
                            <th>Time</th>
                            <th>Throughput</th>
                            <th>Speedup</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>{}</td>
                            <td>{:.2} seconds</td>
                            <td>{:.1} K tri/s</td>
                            <td><span class="speedup speedup-low">1.0× (baseline)</span></td>
                        </tr>
                        <tr>
                            <td>{}</td>
                            <td>{:.2} seconds</td>
                            <td>{:.1} K tri/s</td>
                            <td><span class="speedup speedup-medium">{:.1}×</span></td>
                        </tr>
                        <tr class="winner-row">
                            <td><strong>{}</strong></td>
                            <td><strong>{:.3} seconds</strong></td>
                            <td><strong>{:.1} K tri/s</strong></td>
                            <td><span class="speedup speedup-high">{:.1}×</span></td>
                        </tr>
                    </tbody>
                </table>
            </section>

            <section>
                <h2>Key Findings</h2>
                <div class="highlight-box">
                    <ul style="list-style: none; padding: 0;">
                        <li style="padding: 0.5rem 0;">✅ <strong>{:.1}× faster</strong> - measured speedup over single-mesh</li>
                        <li style="padding: 0.5rem 0;">✅ <strong>{:.1}K → {:.1}K tri/s</strong> - measured throughput increase</li>
                        <li style="padding: 0.5rem 0;">✅ <strong>{:.2}s → {:.0}ms</strong> - measured time reduction</li>
                        <li style="padding: 0.5rem 0;">✅ <strong>{:.1}× faster sequential</strong> - tiling improves cache locality</li>
                    </ul>
                    <p style="margin-top: 1rem; font-style: italic; color: #64748b;">
                        All data measured from Criterion benchmarks on this system. Your results may vary based on CPU cores and system load.
                    </p>
                </div>
            </section>

            <section>
                <h2>Visual Comparison</h2>
                <div class="chart-container">
                    <canvas id="timeChart"></canvas>
                </div>

                <h3 style="color: #764ba2; margin-top: 2rem;">Throughput</h3>
                <div class="chart-container">
                    <canvas id="throughputChart"></canvas>
                </div>

                <h3 style="color: #764ba2; margin-top: 2rem;">Speedup Factor</h3>
                <div class="chart-container">
                    <canvas id="speedupChart"></canvas>
                </div>
            </section>
        </div>

        <footer>
            <p><strong>Rerecast</strong> - Rust port of Recast Navigation</p>
            <p>Generated with 100% Rust 🦀</p>
        </footer>
    </div>

    <script>
        new Chart(document.getElementById('timeChart'), {{
            type: 'bar',
            data: {{
                labels: {},
                datasets: [{{
                    label: 'Time (seconds)',
                    data: {},
                    backgroundColor: ['#64748b', '#f59e0b', '#10b981']
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{ legend: {{ display: false }}, title: {{ display: true, text: 'Lower is Better' }} }},
                scales: {{ y: {{ beginAtZero: true, title: {{ display: true, text: 'Time (seconds)' }} }} }}
            }}
        }});

        new Chart(document.getElementById('throughputChart'), {{
            type: 'bar',
            data: {{
                labels: {},
                datasets: [{{
                    label: 'Throughput (K tri/s)',
                    data: {},
                    backgroundColor: ['#64748b', '#f59e0b', '#10b981']
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{ legend: {{ display: false }}, title: {{ display: true, text: 'Higher is Better' }} }},
                scales: {{ y: {{ beginAtZero: true, title: {{ display: true, text: 'Throughput (K tri/s)' }} }} }}
            }}
        }});

        new Chart(document.getElementById('speedupChart'), {{
            type: 'bar',
            data: {{
                labels: {},
                datasets: [{{
                    label: 'Speedup (×)',
                    data: {},
                    backgroundColor: ['#64748b', '#f59e0b', '#10b981']
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{ legend: {{ display: false }}, title: {{ display: true, text: 'Higher is Better' }} }},
                scales: {{ y: {{ beginAtZero: true, title: {{ display: true, text: 'Speedup (×)' }} }} }}
            }}
        }});
    </script>
</body>
</html>"#,
        speedups[2],
        sorted[0].1.time_s, sorted[2].1.time_s * 1000.0,
        sorted[0].0, sorted[0].1.time_s, sorted[0].1.throughput,
        sorted[1].0, sorted[1].1.time_s, sorted[1].1.throughput,
        sorted[2].0, sorted[2].1.time_s, sorted[2].1.throughput,
        sorted[0].0, sorted[0].1.time_s, sorted[0].1.throughput,
        sorted[1].0, sorted[1].1.time_s, sorted[1].1.throughput, speedups[1],
        sorted[2].0, sorted[2].1.time_s, sorted[2].1.throughput, speedups[2],
        // Key findings
        speedups[2],
        sorted[0].1.throughput, sorted[2].1.throughput,
        sorted[0].1.time_s, sorted[2].1.time_s * 1000.0,
        speedups[1],
        // Charts
        approaches_json, times_json,
        approaches_json, throughputs_json,
        approaches_json, speedups_json,
    )
}

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Rerecast Navmesh Benchmark - Complete Report Generator   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Step 1: Run benchmarks
    run_benchmarks()?;

    // Step 2: Find results
    println!("🔍 Looking for Criterion results...");
    let criterion_dir = find_criterion_dir().context("Failed to find Criterion directory")?;
    println!("✅ Found: {}", criterion_dir.display());

    // Step 3: Parse results
    println!("📊 Parsing benchmark results...");
    let results = parse_benchmark_results(&criterion_dir)?;
    println!("✅ Parsed {} benchmarks", results.len());

    if results.is_empty() {
        anyhow::bail!("No benchmark results found!");
    }

    // Display parsed results
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│ Benchmark Results                                       │");
    println!("└─────────────────────────────────────────────────────────┘");
    for result in &results {
        println!("  {} : {:.3}s ({:.1}K tri/s)", 
            result.name.replace("_", " "), 
            result.time_s, 
            result.throughput
        );
    }

    // Step 4: Generate HTML
    println!("\n📝 Generating HTML report with charts...");
    let html = generate_html(&results);

    // Always write to workspace root docs directory
    // We're running from crates/benchmark-report, so ../../docs is workspace root
    let workspace_root = PathBuf::from("../..");
    let docs_dir = workspace_root.join("docs");
    let output_path = docs_dir.join("benchmark-results.html");
    
    fs::create_dir_all(&docs_dir)?;
    fs::write(&output_path, html)?;

    println!("✅ Report generated: {}", output_path.canonicalize()?.display());
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  ✅ SUCCESS! All benchmarks complete and report generated  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    let canonical_path = output_path.canonicalize()?;
    println!("\n🌐 Report saved to:");
    println!("   {}", canonical_path.display());
    println!("\n📊 The report includes:");
    println!("   • Real measured performance data");
    println!("   • Interactive charts (Chart.js)");
    println!("   • Detailed comparison tables");
    println!("   • All data from actual benchmarks");
    println!("\n💡 To regenerate: cargo run -p benchmark-report --release");
    println!("\n🌐 Opening in browser...");
    
    // Open in browser using the canonical path
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd")
            .args(&["/C", "start", "", &canonical_path.to_string_lossy()])
            .spawn();
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open")
            .arg(&canonical_path)
            .spawn();
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("xdg-open")
            .arg(&canonical_path)
            .spawn();
    }

    Ok(())
}
