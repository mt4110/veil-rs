use std::collections::HashMap;
use veil_core::model::Finding;

use crate::formatters::{Formatter, Summary};
use anyhow::Result;

pub struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn print(&self, findings: &[Finding], _summary: &Summary) -> Result<()> {
        let report = self.generate_report(findings);
        println!("{}", report);
        Ok(())
    }
}

impl HtmlFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(&self, findings: &[Finding]) -> String {
        let now_utc = chrono::Utc::now();
        let generated_at = now_utc.to_rfc3339();

        // Metadata
        let command = std::env::args().collect::<Vec<_>>().join(" ");
        let git_branch = Self::get_git_branch();
        let git_commit = Self::get_git_commit();
        let git_display = match (&git_branch, &git_commit) {
            (Some(br), Some(co)) => format!("{} @ {}", br, co),
            (Some(br), None) => br.clone(),
            _ => "N/A".to_string(),
        };

        let total_findings = findings.len();

        // Compute Summary
        let mut by_severity = HashMap::new();
        let mut by_rule = HashMap::new();
        for f in findings {
            // Better to use buckets consistent with severity_label
            let label = Self::severity_label(f.score);
            *by_severity.entry(label).or_insert(0) += 1;

            *by_rule.entry(f.rule_id.clone()).or_insert(0) += 1;
        }

        let mut top_rules: Vec<_> = by_rule.into_iter().collect();
        top_rules.sort_by(|a, b| b.1.cmp(&a.1));
        top_rules.truncate(3);

        let severity_order = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];
        let severity_summary = severity_order
            .iter()
            .map(|&label| {
                let count = by_severity.get(label).unwrap_or(&0);
                format!("{}: {}", label, count)
            })
            .collect::<Vec<_>>()
            .join(" / ");

        let top_rules_html = top_rules
            .iter()
            .map(|(id, count)| format!("<li>{} ({})</li>", html_escape(id), count))
            .collect::<Vec<_>>()
            .join("");

        let rows = findings
            .iter()
            .map(|f| {
                format!(
                    r#"<tr class="finding-row" data-severity="{}" data-rule-id="{}" data-file-path="{}">
                    <td><span class="badge {}">{}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td class="mono">{}</td>
                    <td>{}</td>
                    <td class="mono">{}</td>
                </tr>"#,
                    Self::severity_label(f.score),
                    html_escape(&f.rule_id),
                    html_escape(&f.path.to_string_lossy()),
                    Self::severity_class(f.score),
                    Self::severity_label(f.score),
                    f.score,
                    html_escape(&f.rule_id),
                    html_escape(&f.path.to_string_lossy()),
                    html_escape(&f.masked_snippet),
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

        #report-meta {{
            margin-bottom: 1rem;
            font-size: 0.9rem;
            color: var(--text-secondary);
        }}

        .meta-line {{ margin-bottom: 0.25rem; }}
        .meta-label {{ font-weight: bold; margin-right: 0.25rem; color: var(--text-primary); }}
        .meta-value code {{ 
            background: #edf2f7; 
            padding: 0.2rem 0.4rem; 
            border-radius: 0.25rem; 
            font-family: monospace;
        }}

        /* Summary Cards */
        #summary-cards {{
            display: flex;
            flex-wrap: wrap;
            gap: 1rem;
            margin-bottom: 2rem;
        }}

        .summary-card {{
            padding: 1rem 1.5rem;
            border-radius: 0.5rem;
            border: 1px solid var(--border-color);
            background: var(--bg-card);
            min-width: 200px;
            box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
            flex: 1;
        }}
        .summary-label {{ font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 0.5rem; text-transform: uppercase; letter-spacing: 0.05em; }}
        .summary-value {{ font-size: 1.25rem; font-weight: 700; color: var(--text-primary); }}
        
        .summary-list {{
            margin: 0;
            padding-left: 1.2rem;
            font-size: 0.9rem;
        }}
        .summary-list li {{ margin-bottom: 0.25rem; }}

        /* Filters */
        #filters {{
            background: var(--bg-card);
            padding: 1rem;
            border-radius: 0.5rem;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
            margin-bottom: 1rem;
            display: flex;
            gap: 2rem;
            align-items: center;
            flex-wrap: wrap;
        }}

        .filter-group {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .filter-group span {{
            font-weight: 600;
            font-size: 0.875rem;
            color: var(--text-secondary);
            margin-right: 0.5rem;
        }}

        .filter-group label {{
            display: flex;
            align-items: center;
            gap: 0.25rem;
            font-size: 0.875rem;
            cursor: pointer;
        }}

        #search-input {{
            padding: 0.5rem;
            border: 1px solid var(--border-color);
            border-radius: 0.25rem;
            min-width: 300px;
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
            </div>
            <div>
                <!-- Right side header content if needed -->
            </div>
        </header>

        <div id="report-meta">
            <div class="meta-line">
                <span class="meta-label">Scanned at:</span>
                <span class="meta-value">{generated_at}</span>
            </div>
            <div class="meta-line">
                <span class="meta-label">Command:</span>
                <span class="meta-value"><code>{command}</code></span>
            </div>
            <div class="meta-line">
                <span class="meta-label">Git:</span>
                <span class="meta-value">{git_info}</span>
            </div>
        </div>

        <section id="summary-cards">
            <div class="summary-card">
                <div class="summary-label">Total Findings</div>
                <div class="summary-value">{total}</div>
            </div>
            <div class="summary-card">
                <div class="summary-label">By Severity</div>
                <div class="summary-value" style="font-size: 1rem;">
                    {severity_summary}
                </div>
            </div>
            <div class="summary-card">
                <div class="summary-label">Top Rules</div>
                <ul class="summary-list">
                    {top_rules}
                </ul>
            </div>
        </section>

        <div id="filters">
            <div class="filter-group">
                <span>Severity:</span>
                <label><input type="checkbox" name="severity" value="LOW" checked> LOW</label>
                <label><input type="checkbox" name="severity" value="MEDIUM" checked> MEDIUM</label>
                <label><input type="checkbox" name="severity" value="HIGH" checked> HIGH</label>
                <label><input type="checkbox" name="severity" value="CRITICAL" checked> CRITICAL</label>
            </div>
        
            <div class="filter-group">
                <span>Search:</span>
                <input id="search-input" type="text" placeholder="Filter by rule ID or file path...">
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

    <script>
      (function() {{
        const rows = Array.from(document.querySelectorAll(".finding-row"));
        const checkboxes = Array.from(document.querySelectorAll("input[name='severity']"));
        const searchInput = document.getElementById("search-input");

        function applyFilters() {{
          const activeSeverities = new Set(
            checkboxes.filter(cb => cb.checked).map(cb => cb.value.toUpperCase())
          );
          const query = (searchInput.value || "").toLowerCase().trim();

          rows.forEach(row => {{
            const sev = (row.dataset.severity || "").toUpperCase();
            const ruleId = (row.dataset.ruleId || "").toLowerCase();
            const filePath = (row.dataset.filePath || "").toLowerCase();

            let visible = true;

            if (activeSeverities.size > 0 && !activeSeverities.has(sev)) {{
              visible = false;
            }}

            if (query) {{
              const haystack = ruleId + " " + filePath;
              if (!haystack.includes(query)) {{
                visible = false;
              }}
            }}

            row.style.display = visible ? "" : "none";
          }});
        }}

        checkboxes.forEach(cb => cb.addEventListener("change", applyFilters));
        if (searchInput) {{
          searchInput.addEventListener("input", function() {{
            applyFilters();
          }});
        }}

        applyFilters(); 
      }})();
    </script>
</body>
</html>
"#,
            generated_at = generated_at,
            command = html_escape(&command),
            git_info = html_escape(&git_display),
            total = total_findings,
            severity_summary = severity_summary,
            top_rules = top_rules_html,
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

    fn get_git_branch() -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    fn get_git_commit() -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use veil_core::model::{Finding, Severity};

    #[test]
    fn test_html_generation() {
        let formatter = HtmlFormatter::new();
        let findings = vec![Finding {
            path: PathBuf::from("test.txt"),
            line_number: 1,
            line_content: "secret=123".to_string(),
            matched_content: "password".to_string(),
            masked_snippet: "********".to_string(),
            rule_id: "test_rule".to_string(),
            severity: Severity::High,
            score: 80,
            grade: veil_core::rules::grade::Grade::Critical,
            context_before: vec![],
            context_after: vec![],
            commit_sha: None,
            author: None,
            date: None,
        }];

        let report = formatter.generate_report(&findings);
        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("test.txt"));
        assert!(report.contains("test_rule"));
        assert!(report.contains("********"));
    }
}
