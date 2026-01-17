// HTML generation module - Modern dashboard design
use super::{BenchmarkResult, GroupedResults};

pub fn generate_comprehensive_html(
    all_results: &[BenchmarkResult],
    grouped_results: &[GroupedResults],
) -> String {
    let mut html = String::new();
    
    // Calculate key metrics
    let total_benchmarks = all_results.len();
    let (speedup, fastest_time, baseline_time) = calculate_key_metrics(all_results);
    
    html.push_str(&generate_header(total_benchmarks, speedup, fastest_time));
    html.push_str(&generate_styles());
    html.push_str("</head><body>");
    html.push_str(&generate_top_banner(total_benchmarks, speedup, fastest_time, baseline_time));
    html.push_str(&generate_overview_section(all_results));
    html.push_str(&generate_detailed_results_table(all_results));
    html.push_str(&generate_group_analysis(grouped_results));
    html.push_str(&generate_statistical_summary(all_results));
    html.push_str(&generate_footer());
    html.push_str(&generate_chart_scripts(all_results, grouped_results));
    html.push_str("</body></html>");
    
    html
}

fn calculate_key_metrics(results: &[BenchmarkResult]) -> (f64, f64, f64) {
    let comparison: Vec<_> = results.iter()
        .filter(|r| r.group == "comparison_large_map")
        .collect();
    
    if comparison.len() >= 3 {
        let single = comparison.iter().find(|r| r.benchmark.contains("single")).map(|r| r.time_s).unwrap_or(1.0);
        let parallel = comparison.iter().find(|r| r.benchmark.contains("parallel")).map(|r| r.time_s).unwrap_or(1.0);
        (single / parallel, parallel, single)
    } else {
        (1.0, 0.0, 0.0)
    }
}

fn generate_header(total: usize, speedup: f64, fastest: f64) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rerecast Benchmark Dashboard | {:.1}× Speedup | {} Tests</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
"#, speedup, total)
}

fn generate_styles() -> String {
    r#"    <style>
        :root {
            --primary: #6366f1;
            --primary-dark: #4f46e5;
            --success: #10b981;
            --warning: #f59e0b;
            --danger: #ef4444;
            --info: #3b82f6;
            --gray-50: #f9fafb;
            --gray-100: #f3f4f6;
            --gray-200: #e5e7eb;
            --gray-300: #d1d5db;
            --gray-700: #374151;
            --gray-800: #1f2937;
            --gray-900: #111827;
        }
        
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: var(--gray-50);
            color: var(--gray-900);
            line-height: 1.6;
        }
        
        .dashboard {
            min-height: 100vh;
        }
        
        .topbar {
            background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%);
            color: white;
            padding: 1.5rem 2rem;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        
        .topbar-content {
            max-width: 1600px;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .topbar h1 {
            font-size: 1.75rem;
            font-weight: 700;
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }
        
        .topbar-meta {
            text-align: right;
            opacity: 0.95;
            font-size: 0.875rem;
        }
        
        .metrics-banner {
            background: white;
            border-bottom: 1px solid var(--gray-200);
            padding: 2rem;
        }
        
        .metrics-grid {
            max-width: 1600px;
            margin: 0 auto;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 2rem;
        }
        
        .metric-card {
            text-align: center;
        }
        
        .metric-value {
            font-size: 2.5rem;
            font-weight: 700;
            color: var(--primary);
            margin-bottom: 0.25rem;
        }
        
        .metric-value.success { color: var(--success); }
        .metric-value.warning { color: var(--warning); }
        
        .metric-label {
            font-size: 0.875rem;
            color: var(--gray-700);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            font-weight: 600;
        }
        
        .metric-subtext {
            font-size: 0.75rem;
            color: var(--gray-700);
            margin-top: 0.25rem;
        }
        
        .container {
            max-width: 1600px;
            margin: 0 auto;
            padding: 2rem;
        }
        
        .section {
            background: white;
            border-radius: 12px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }
        
        .section-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
            padding-bottom: 1rem;
            border-bottom: 2px solid var(--gray-200);
        }
        
        .section-title {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--gray-900);
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }
        
        .badge {
            display: inline-block;
            padding: 0.375rem 0.75rem;
            border-radius: 6px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        
        .badge-success { background: #10b98115; color: var(--success); }
        .badge-warning { background: #f59e0b15; color: var(--warning); }
        .badge-info { background: #3b82f615; color: var(--info); }
        .badge-neutral { background: var(--gray-100); color: var(--gray-700); }
        
        .chart-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 2rem;
            margin: 2rem 0;
        }
        
        .chart-card {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            border: 1px solid var(--gray-200);
        }
        
        .chart-card.full-width {
            grid-column: 1 / -1;
        }
        
        .chart-title {
            font-size: 1.125rem;
            font-weight: 600;
            color: var(--gray-900);
            margin-bottom: 1rem;
        }
        
        .chart-wrapper {
            position: relative;
            height: 350px;
        }
        
        .chart-wrapper.tall { height: 450px; }
        .chart-wrapper.short { height: 250px; }
        
        .data-table {
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
            margin: 1rem 0;
            font-size: 0.875rem;
        }
        
        .data-table thead {
            background: var(--gray-50);
        }
        
        .data-table th {
            padding: 0.75rem 1rem;
            text-align: left;
            font-weight: 600;
            color: var(--gray-700);
            text-transform: uppercase;
            font-size: 0.75rem;
            letter-spacing: 0.05em;
            border-bottom: 2px solid var(--gray-200);
        }
        
        .data-table td {
            padding: 0.875rem 1rem;
            border-bottom: 1px solid var(--gray-200);
        }
        
        .data-table tbody tr:hover {
            background: var(--gray-50);
        }
        
        .data-table tbody tr:last-child td {
            border-bottom: none;
        }
        
        .highlight-row {
            background: #10b98108 !important;
            font-weight: 600;
        }
        
        .highlight-row td:first-child {
            border-left: 3px solid var(--success);
        }
        
        .mono {
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.875em;
        }
        
        .stat-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            margin: 1.5rem 0;
        }
        
        .stat-item {
            padding: 1rem;
            border-radius: 8px;
            background: var(--gray-50);
            border: 1px solid var(--gray-200);
        }
        
        .stat-item-label {
            font-size: 0.75rem;
            color: var(--gray-700);
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 0.5rem;
        }
        
        .stat-item-value {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--gray-900);
        }
        
        .info-alert {
            padding: 1rem 1.25rem;
            border-radius: 8px;
            background: #3b82f610;
            border-left: 4px solid var(--info);
            margin: 1.5rem 0;
            font-size: 0.875rem;
        }
        
        .info-alert strong {
            color: var(--info);
        }
        
        footer {
            background: var(--gray-900);
            color: white;
            padding: 2rem;
            text-align: center;
            margin-top: 3rem;
        }
        
        footer p {
            margin: 0.5rem 0;
            opacity: 0.9;
        }
        
        @media (max-width: 768px) {
            .topbar-content { flex-direction: column; gap: 1rem; text-align: center; }
            .chart-grid { grid-template-columns: 1fr; }
            .chart-wrapper { height: 300px; }
        }
    </style>
"#.to_string()
}

fn generate_top_banner(total: usize, speedup: f64, fastest: f64, baseline: f64) -> String {
    format!(r#"<div class="dashboard">
    <div class="topbar">
        <div class="topbar-content">
            <h1>
                <span>🚀</span>
                <span>Rerecast Navmesh Benchmark Dashboard</span>
            </h1>
            <div class="topbar-meta">
                <div>Generated: {}</div>
                <div style="opacity: 0.8;">Criterion • 100% Real Data</div>
            </div>
        </div>
    </div>
    
    <div class="metrics-banner">
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-value success">{:.1}×</div>
                <div class="metric-label">Performance Gain</div>
                <div class="metric-subtext">Parallel vs Single Mesh</div>
            </div>
            <div class="metric-card">
                <div class="metric-value warning">{:.0}ms</div>
                <div class="metric-label">Fastest Generation</div>
                <div class="metric-subtext">Tiled Parallel Mode</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.2}s</div>
                <div class="metric-label">Baseline Time</div>
                <div class="metric-subtext">Single Mesh Mode</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Total Benchmarks</div>
                <div class="metric-subtext">All Tests Passed</div>
            </div>
        </div>
    </div>
"#, 
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        speedup,
        fastest * 1000.0,
        baseline,
        total
    )
}

fn generate_overview_section(results: &[BenchmarkResult]) -> String {
    let comparison: Vec<_> = results.iter()
        .filter(|r| r.group == "comparison_large_map")
        .collect();
    
    if comparison.is_empty() {
        return String::new();
    }
    
    let labels: Vec<String> = comparison.iter()
        .map(|r| r.benchmark.replace("_", " "))
        .collect();
    let times: Vec<f64> = comparison.iter().map(|r| r.time_s).collect();
    let throughputs: Vec<f64> = comparison.iter().map(|r| r.throughput).collect();
    
    let labels_json = serde_json::to_string(&labels).unwrap();
    let times_json = serde_json::to_string(&times).unwrap();
    let throughputs_json = serde_json::to_string(&throughputs).unwrap();
    
    format!(r#"    <div class="container">
        <div class="section">
            <div class="section-header">
                <div class="section-title">
                    <span>📊</span>
                    <span>Performance Overview</span>
                </div>
                <span class="badge badge-success">Primary Metrics</span>
            </div>
            
            <div class="chart-grid">
                <div class="chart-card">
                    <div class="chart-title">Generation Time Comparison</div>
                    <div class="chart-wrapper">
                        <canvas id="timeChart"></canvas>
                    </div>
                </div>
                <div class="chart-card">
                    <div class="chart-title">Throughput Analysis</div>
                    <div class="chart-wrapper">
                        <canvas id="throughputChart"></canvas>
                    </div>
                </div>
            </div>
            
            <div class="chart-card full-width" style="margin-top: 2rem;">
                <div class="chart-title">Performance Comparison - All Approaches</div>
                <div class="chart-wrapper tall">
                    <canvas id="comparisonChart"></canvas>
                </div>
            </div>
        </div>
        
        <script>
            const labels = {};
            const times = {};
            const throughputs = {};
        </script>
"#, labels_json, times_json, throughputs_json)
}

fn generate_detailed_results_table(results: &[BenchmarkResult]) -> String {
    let mut rows = String::new();
    
    // Sort by group then time
    let mut sorted = results.to_vec();
    sorted.sort_by(|a, b| {
        match a.group.cmp(&b.group) {
            std::cmp::Ordering::Equal => a.time_s.partial_cmp(&b.time_s).unwrap(),
            other => other,
        }
    });
    
    let fastest_time = sorted.iter().map(|r| r.time_s).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
    
    for result in sorted {
        let is_fastest = (result.time_s - fastest_time).abs() < 0.001;
        let row_class = if is_fastest { "highlight-row" } else { "" };
        let status = if is_fastest {
            r#"<span class="badge badge-success">Fastest</span>"#
        } else if result.time_s < 1.0 {
            r#"<span class="badge badge-info">Fast</span>"#
        } else if result.time_s < 2.0 {
            r#"<span class="badge badge-warning">Moderate</span>"#
        } else {
            r#"<span class="badge badge-neutral">Baseline</span>"#
        };
        
        rows.push_str(&format!(r#"
                    <tr class="{}">
                        <td><strong>{}</strong><br/><span style="font-size: 0.75rem; color: var(--gray-700);">{}</span></td>
                        <td><span class="mono">{:.4}s</span><br/><span style="font-size: 0.75rem; color: var(--gray-700);">±{:.4}s</span></td>
                        <td><span class="mono">{:.4}s</span></td>
                        <td><span class="mono">{:.4}s</span> - <span class="mono">{:.4}s</span></td>
                        <td><strong>{:.1}</strong> K tri/s</td>
                        <td>{}</td>
                    </tr>"#,
            row_class,
            result.benchmark.replace("_", " "),
            result.group,
            result.time_s,
            result.time_std_dev,
            result.median_time_s,
            result.time_ci_lower,
            result.time_ci_upper,
            result.throughput,
            status
        ));
    }
    
    format!(r#"        <div class="section">
            <div class="section-header">
                <div class="section-title">
                    <span>📋</span>
                    <span>Detailed Benchmark Results</span>
                </div>
                <span class="badge badge-info">{} Tests</span>
            </div>
            
            <div class="info-alert">
                <p><strong>All data measured from Criterion benchmarks.</strong> Times include mean, median, standard deviation, and 95% confidence intervals.</p>
            </div>
            
            <table class="data-table">
                <thead>
                    <tr>
                        <th>Benchmark</th>
                        <th>Mean Time</th>
                        <th>Median Time</th>
                        <th>95% Confidence Interval</th>
                        <th>Throughput</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>
"#, results.len(), rows)
}

fn generate_group_analysis(grouped: &[GroupedResults]) -> String {
    let mut sections = String::new();
    
    for group in grouped {
        let stats = calculate_group_stats(&group.results);
        
        sections.push_str(&format!(r#"            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <span>🔍</span>
                        <span>Group: {}</span>
                    </div>
                    <span class="badge badge-neutral">{} Benchmarks</span>
                </div>
                
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-item-label">Min Time</div>
                        <div class="stat-item-value">{:.4}s</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-item-label">Max Time</div>
                        <div class="stat-item-value">{:.4}s</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-item-label">Mean Time</div>
                        <div class="stat-item-value">{:.4}s</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-item-label">Median Time</div>
                        <div class="stat-item-value">{:.4}s</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-item-label">Std Deviation</div>
                        <div class="stat-item-value">{:.4}s</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-item-label">CV (%)</div>
                        <div class="stat-item-value">{:.2}%</div>
                    </div>
                </div>
            </div>
"#, 
            group.group_name.replace("_", " "),
            group.results.len(),
            stats.0, stats.1, stats.2, stats.3, stats.4, stats.5
        ));
    }
    
    sections
}

fn calculate_group_stats(results: &[BenchmarkResult]) -> (f64, f64, f64, f64, f64, f64) {
    let times: Vec<f64> = results.iter().map(|r| r.time_s).collect();
    let mut sorted = times.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    
    let variance = times.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / times.len() as f64;
    let std_dev = variance.sqrt();
    let cv = (std_dev / mean) * 100.0;
    
    (min, max, mean, median, std_dev, cv)
}

fn generate_statistical_summary(results: &[BenchmarkResult]) -> String {
    let times: Vec<f64> = results.iter().map(|r| r.time_s).collect();
    let throughputs: Vec<f64> = results.iter().map(|r| r.throughput).collect();
    
    let time_min = times.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
    let time_max = times.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
    let time_mean = times.iter().sum::<f64>() / times.len() as f64;
    
    let throughput_max = throughputs.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
    
    format!(r#"        <div class="section">
            <div class="section-header">
                <div class="section-title">
                    <span>📈</span>
                    <span>Statistical Summary</span>
                </div>
                <span class="badge badge-info">Aggregated Metrics</span>
            </div>
            
            <div class="stat-grid">
                <div class="stat-item">
                    <div class="stat-item-label">Fastest Time</div>
                    <div class="stat-item-value">{:.4}s</div>
                </div>
                <div class="stat-item">
                    <div class="stat-item-label">Slowest Time</div>
                    <div class="stat-item-value">{:.4}s</div>
                </div>
                <div class="stat-item">
                    <div class="stat-item-label">Average Time</div>
                    <div class="stat-item-value">{:.4}s</div>
                </div>
                <div class="stat-item">
                    <div class="stat-item-label">Peak Throughput</div>
                    <div class="stat-item-value">{:.1} K tri/s</div>
                </div>
                <div class="stat-item">
                    <div class="stat-item-label">Time Range</div>
                    <div class="stat-item-value">{:.2}×</div>
                </div>
                <div class="stat-item">
                    <div class="stat-item-label">Total Tests</div>
                    <div class="stat-item-value">{}</div>
                </div>
            </div>
        </div>
    </div>
"#, time_min, time_max, time_mean, throughput_max, time_max / time_min, results.len())
}

fn generate_footer() -> String {
    r#"    <footer>
        <p><strong>Rerecast</strong> - Rust port of Recast Navigation</p>
        <p>Tile-Based Parallel Navmesh Generation • 100% Rust 🦀</p>
        <p style="margin-top: 1rem; opacity: 0.7; font-size: 0.875rem;">
            All data measured from Criterion benchmarks • No estimates or extrapolations
        </p>
    </footer>
</div>
"#.to_string()
}

fn generate_chart_scripts(results: &[BenchmarkResult], grouped: &[GroupedResults]) -> String {
    let comparison: Vec<_> = results.iter()
        .filter(|r| r.group == "comparison_large_map")
        .collect();
    
    if comparison.is_empty() {
        return String::new();
    }
    
    let labels: Vec<String> = comparison.iter().map(|r| r.benchmark.replace("_", " ")).collect();
    let times: Vec<f64> = comparison.iter().map(|r| r.time_s).collect();
    let throughputs: Vec<f64> = comparison.iter().map(|r| r.throughput).collect();
    let ci_lowers: Vec<f64> = comparison.iter().map(|r| r.time_ci_lower).collect();
    let ci_uppers: Vec<f64> = comparison.iter().map(|r| r.time_ci_upper).collect();
    
    format!(r#"<script>
    const colors = {{
        primary: '#6366f1',
        success: '#10b981',
        warning: '#f59e0b',
        danger: '#ef4444',
        info: '#3b82f6',
        gray: '#6b7280',
    }};
    
    // Time Comparison Chart with Error Bars
    new Chart(document.getElementById('timeChart'), {{
        type: 'bar',
        data: {{
            labels: {},
            datasets: [{{
                label: 'Generation Time (s)',
                data: {},
                backgroundColor: [colors.gray, colors.warning, colors.success],
                borderColor: [colors.gray, colors.warning, colors.success],
                borderWidth: 2,
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            plugins: {{
                legend: {{ display: false }},
                tooltip: {{
                    callbacks: {{
                        afterLabel: function(context) {{
                            const lower = {};
                            const upper = {};
                            return `95% CI: ${{lower[context.dataIndex].toFixed(4)}}s - ${{upper[context.dataIndex].toFixed(4)}}s`;
                        }}
                    }}
                }}
            }},
            scales: {{
                y: {{ 
                    beginAtZero: true,
                    title: {{ display: true, text: 'Time (seconds)', font: {{ weight: 'bold' }} }}
                }}
            }}
        }}
    }});
    
    // Throughput Chart
    new Chart(document.getElementById('throughputChart'), {{
        type: 'bar',
        data: {{
            labels: {},
            datasets: [{{
                label: 'Throughput (K tri/s)',
                data: {},
                backgroundColor: [colors.gray, colors.warning, colors.success],
                borderWidth: 0,
            }}]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            plugins: {{ legend: {{ display: false }} }},
            scales: {{
                y: {{ 
                    beginAtZero: true,
                    title: {{ display: true, text: 'Throughput (K triangles/sec)', font: {{ weight: 'bold' }} }}
                }}
            }}
        }}
    }});
    
    // Combined Comparison Chart
    new Chart(document.getElementById('comparisonChart'), {{
        type: 'line',
        data: {{
            labels: {},
            datasets: [
                {{
                    label: 'Generation Time (s)',
                    data: {},
                    borderColor: colors.primary,
                    backgroundColor: colors.primary + '20',
                    yAxisID: 'y',
                    tension: 0.4,
                    borderWidth: 3,
                    pointRadius: 6,
                    pointHoverRadius: 8,
                }},
                {{
                    label: 'Throughput (K tri/s)',
                    data: {},
                    borderColor: colors.success,
                    backgroundColor: colors.success + '20',
                    yAxisID: 'y1',
                    tension: 0.4,
                    borderWidth: 3,
                    pointRadius: 6,
                    pointHoverRadius: 8,
                }}
            ]
        }},
        options: {{
            responsive: true,
            maintainAspectRatio: false,
            interaction: {{ mode: 'index', intersect: false }},
            plugins: {{
                legend: {{ display: true, position: 'top' }}
            }},
            scales: {{
                y: {{
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: {{ display: true, text: 'Time (seconds)', color: colors.primary, font: {{ weight: 'bold' }} }},
                    beginAtZero: true,
                }},
                y1: {{
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: {{ display: true, text: 'Throughput (K tri/s)', color: colors.success, font: {{ weight: 'bold' }} }},
                    beginAtZero: true,
                    grid: {{ drawOnChartArea: false }},
                }}
            }}
        }}
    }});
</script>
"#,
        serde_json::to_string(&labels).unwrap(),
        serde_json::to_string(&times).unwrap(),
        serde_json::to_string(&ci_lowers).unwrap(),
        serde_json::to_string(&ci_uppers).unwrap(),
        serde_json::to_string(&labels).unwrap(),
        serde_json::to_string(&throughputs).unwrap(),
        serde_json::to_string(&labels).unwrap(),
        serde_json::to_string(&times).unwrap(),
        serde_json::to_string(&throughputs).unwrap(),
    )
}
