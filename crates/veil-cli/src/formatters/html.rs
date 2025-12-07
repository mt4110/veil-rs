use chrono::Local;
use std::collections::HashMap;
use veil_core::model::Finding;

pub struct HtmlFormatter;

impl HtmlFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(&self, findings: &[Finding]) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let total_findings = findings.len();

        let mut severity_counts = HashMap::new();
        for f in findings {
            *severity_counts.entry(f.rule_id.clone()).or_insert(0) += 1;
        }

        // Simple severity breakdown
        let critical_count = findings.iter().filter(|f| f.score >= 90).count();
        let high_count = findings
            .iter()
            .filter(|f| f.score >= 70 && f.score < 90)
            .count();
        let medium_count = findings
            .iter()
            .filter(|f| f.score >= 40 && f.score < 70)
            .count();
        let low_count = findings.iter().filter(|f| f.score < 40).count();

        let rows = findings
            .iter()
            .map(|f| {
                format!(
                    r#"<tr>
                    <td><span class="badge {}">{}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td class="mono">{}</td>
                    <td>{}</td>
                    <td class="mono">{}</td>
                </tr>"#,
                    Self::severity_class(f.score),
                    Self::severity_label(f.score),
                    f.score,
                    html_escape(&f.rule_id),
                    html_escape(&f.path.to_string_lossy()),
                    html_escape(&f.line_content), // Note: In real app, might want to limit length
                    f.line_number
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Veil Security Report</title>
    <style>
        :root {{
            --bg-primary: #f8f9fa;
            --bg-card: #ffffff;
            --text-primary: #2d3748;
            --text-secondary: #718096;
            --border-color: #e2e8f0;
            --accent-color: #4c51bf;
            
            --sev-critical: #e53e3e;
            --sev-high: #dd6b20;
            --sev-medium: #d69e2e;
            --sev-low: #3182ce;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif;
            background-color: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.5;
            margin: 0;
            padding: 2rem;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}

        header {{
            margin-bottom: 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        h1 {{
            font-size: 1.875rem;
            font-weight: 700;
            color: var(--text-primary);
            margin: 0;
        }}

        .meta {{
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}

        .stat-card {{
            background: var(--bg-card);
            padding: 1.5rem;
            border-radius: 0.5rem;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
            text-align: center;
        }}

        .stat-value {{
            font-size: 2rem;
            font-weight: 700;
            line-height: 1;
            margin-bottom: 0.5rem;
        }}

        .stat-label {{
            color: var(--text-secondary);
            font-size: 0.875rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .finding-table {{
            width: 100%;
            background: var(--bg-card);
            border-radius: 0.5rem;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
            overflow: hidden;
            border-collapse: collapse;
        }}

        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--border-color);
        }}

        th {{
            background-color: #edf2f7;
            font-weight: 600;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
        }}

        tr:last-child td {{
            border-bottom: none;
        }}

        .badge {{
            display: inline-block;
            padding: 0.25rem 0.5rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 700;
            color: white;
        }}

        .bg-critical {{ background-color: var(--sev-critical); }}
        .bg-high {{ background-color: var(--sev-high); }}
        .bg-medium {{ background-color: var(--sev-medium); }}
        .bg-low {{ background-color: var(--sev-low); }}

        .mono {{
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace;
            font-size: 0.875rem;
        }}

    </style>
</head>
<body>
    <div class="container">
        <header>
            <div>
                <h1>Veil Security Report</h1>
                <div class="meta">Generated: {now}</div>
            </div>
            <div class="meta">
                Tool: veil-rs <br>
                Status: Complete
            </div>
        </header>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value" style="color: var(--text-primary)">{total}</div>
                <div class="stat-label">Total Findings</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: var(--sev-critical)">{critical}</div>
                <div class="stat-label">Critical</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: var(--sev-high)">{high}</div>
                <div class="stat-label">High</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: var(--sev-medium)">{medium}</div>
                <div class="stat-label">Medium</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: var(--sev-low)">{low}</div>
                <div class="stat-label">Low</div>
            </div>
        </div>

        <table class="finding-table">
            <thead>
                <tr>
                    <th>Severity</th>
                    <th>Score</th>
                    <th>Rule ID</th>
                    <th>File</th>
                    <th>Match Content</th>
                    <th>Line</th>
                </tr>
            </thead>
            <tbody>
                {rows}
            </tbody>
        </table>
    </div>
</body>
</html>
"#,
            now = now,
            total = total_findings,
            critical = critical_count,
            high = high_count,
            medium = medium_count,
            low = low_count,
            rows = rows
        )
    }

    fn severity_class(score: u32) -> &'static str {
        if score >= 90 {
            "bg-critical"
        } else if score >= 70 {
            "bg-high"
        } else if score >= 40 {
            "bg-medium"
        } else {
            "bg-low"
        }
    }

    fn severity_label(score: u32) -> &'static str {
        if score >= 90 {
            "CRITICAL"
        } else if score >= 70 {
            "HIGH"
        } else if score >= 40 {
            "MEDIUM"
        } else {
            "LOW"
        }
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}
