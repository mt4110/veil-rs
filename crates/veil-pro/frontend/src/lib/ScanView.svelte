<script lang="ts">
  import { FolderSearch, AlertTriangle, ShieldCheck, Play, Loader2, RotateCcw } from 'lucide-svelte';
  
  // Strict State Machine for UI prediction and tracking (no implicit states)
  type ScanState = 'Idle' | 'Running' | 'SuccessNoFindings' | 'Violation' | 'Incomplete' | 'ErrorAuth' | 'ErrorConfig' | 'ErrorExpired' | 'ErrorOOM' | 'ErrorUnknown';
  let currentState = $state<ScanState>('Idle');
  
  let scanPath = $state('');
  let findings = $state<any[]>([]);
  let scanStats = $state<{scanned: number, skipped: number, runId: string} | null>(null);
  let errorMsg = $state<string | null>(null);

  const sevWeights: Record<string, number> = { critical: 4, high: 3, medium: 2, low: 1 };
  
  let sortedFindings = $derived(
    [...findings].sort((a, b) => {
      const weightA = sevWeights[String(a.severity || '').toLowerCase()] || 0;
      const weightB = sevWeights[String(b.severity || '').toLowerCase()] || 0;
      return weightB - weightA;
    })
  );

  function safeInt(v: any, fallback = 0): number {
    const n = Number.parseInt(String(v), 10);
    return Number.isFinite(n) ? n : fallback;
  }

  async function triggerScan() {
    currentState = 'Running';
    errorMsg = null;
    scanStats = null;
    findings = [];
    
    try {
      const targetPath = scanPath.trim() === '' ? '.' : scanPath.trim();
      
      const headers: Record<string, string> = {
        'Content-Type': 'application/json'
      };
      
      const token = sessionStorage.getItem('veil_token');
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }

      const res = await fetch('/api/scan', {
        method: 'POST',
        headers,
        body: JSON.stringify({ paths: [targetPath] })
      });

      if (res.status === 401) {
        currentState = 'ErrorAuth';
        errorMsg = 'Session expired or token invalid. Please login again.';
        return;
      }
      if (res.status === 400 || res.status === 422) {
        currentState = 'ErrorConfig';
        const err = await res.json().catch(() => ({}));
        errorMsg = err.error || 'Configuration error or invalid path.';
        return;
      }
      if (res.status === 413) {
        currentState = 'ErrorOOM';
        errorMsg = 'Target is too large. Resource limit exceeded.';
        return;
      }
      if (!res.ok) {
        const err = await res.json().catch(() => ({}));
        errorMsg = err.error || `HTTP ${res.status}`;
        currentState = 'ErrorUnknown';
        return;
      }

      const data = await res.json();
      findings = data.findings || [];
      scanStats = {
        scanned: data.scanned_files,
        skipped: data.skipped_files,
        runId: data.run_id
      };
      
      if (data.limit_reached) {
        currentState = 'Incomplete';
      } else if (findings.length > 0) {
        currentState = 'Violation';
      } else {
        currentState = 'SuccessNoFindings';
      }
      
    } catch (err: any) {
      currentState = 'ErrorUnknown';
      errorMsg = err.message || 'An unknown network error occurred';
    }
  }

  async function downloadEvidence(runId: string) {
    try {
      const headers: Record<string, string> = {};
      const token = sessionStorage.getItem('veil_token');
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
      
      const res = await fetch(`/api/runs/${runId}/evidence.zip`, {
        headers
      });
      
      if (res.status === 410 || res.status === 404) {
        currentState = 'ErrorExpired';
        errorMsg = 'Evidence pack has expired or was discarded. Please run a new scan.';
        return;
      }
      if (res.status === 401) {
        currentState = 'ErrorAuth';
        errorMsg = 'Session expired. Please login again.';
        return;
      }
      if (!res.ok) {
        currentState = 'ErrorUnknown';
        errorMsg = `Failed to download evidence pack (HTTP ${res.status}).`;
        return;
      }
      
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `evidence-${runId}.zip`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
    } catch (e: any) {
      currentState = 'ErrorUnknown';
      errorMsg = 'Network error downloading evidence.';
    }
  }
</script>

<div class="scan-container state-anim">
  <div class="glass-panel input-card">
    <div class="card-header">
      <FolderSearch size={24} color="var(--accent)" />
      <h3>Target Directory</h3>
    </div>
    <form onsubmit={(e) => { e.preventDefault(); triggerScan(); }} class="scan-form">
      <input 
        type="text" 
        bind:value={scanPath} 
        placeholder="Enter path (e.g., /src or default '.') " 
        disabled={currentState === 'Running'}
      />
      <button type="submit" class="btn btn-primary" disabled={currentState === 'Running'}>
        {#if currentState === 'Running'}
          <Loader2 size={18} class="spin" /> Scanning...
        {:else}
          <Play size={18} /> Run Scan
        {/if}
      </button>
    </form>
    
    {#if currentState.startsWith('Error') || currentState === 'Incomplete'}
      <div class="error-banner state-anim">
        <AlertTriangle size={18} /> 
        <div class="error-content">
          <span>{errorMsg || (currentState === 'Incomplete' ? 'Scan limit reached. Results are partial.' : 'An error occurred.')}</span>
          {#if currentState === 'ErrorExpired' || currentState === 'ErrorUnknown' || currentState === 'Incomplete' || currentState === 'ErrorConfig'}
             <button class="btn btn-sm btn-secondary rescan-btn" onclick={triggerScan} type="button">
               <RotateCcw size={14} /> Re-Scan Now
             </button>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if currentState === 'SuccessNoFindings' || currentState === 'Violation' || currentState === 'Incomplete'}
    {#if scanStats}
    <div class="results-area state-anim">
      <div class="stats-cards">
        <div class="stat-card glass-panel">
          <span class="stat-value">{safeInt(scanStats.scanned)}</span>
          <span class="stat-label">Files Scanned</span>
        </div>
        <div class="stat-card glass-panel">
          <span class="stat-value">{safeInt(findings.length)}</span>
          <span class="stat-label">Active Findings</span>
        </div>
        {#if currentState === 'SuccessNoFindings'}
          <div class="stat-card glass-panel success-card">
            <ShieldCheck size={32} color="var(--success)" />
            <span class="stat-label">Zero Violations</span>
          </div>
        {/if}
      </div>

      {#if sortedFindings.length > 0}
        <div class="findings-table glass-panel">
          <div class="table-header">
            <h4>Detected Violations</h4>
            <button class="export-link btn btn-secondary btn-sm" onclick={() => downloadEvidence(scanStats!.runId)} type="button">
              Export Evidence Pack
            </button>
          </div>
          
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Severity</th>
                  <th>Rule</th>
                  <th>Target File</th>
                  <th>Masked Context</th>
                </tr>
              </thead>
              <tbody>
                {#each sortedFindings as finding}
                  <tr>
                    <td>
                      <!-- Strict CSP: Use CSS classes instead of inline style -->
                      <span class="badge sev-{String(finding.severity || '').toLowerCase()}">
                        {finding.severity}
                      </span>
                    </td>
                    <td class="rule-col">{finding.rule_id}</td>
                    <td class="path-col">{finding.path}:{safeInt(finding.line_number)}</td>
                    <td class="context-col">
                      <code class="code-block">{finding.masked_snippet}</code>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    </div>
    {/if}
  {/if}
</div>

<style>
  /* Strict CSP Architecture: Svelte transitions are banned. All motion is CSS. */
  .state-anim {
    animation: fadeIn 0.15s ease-out forwards;
  }
  
  @media (prefers-reduced-motion: reduce) {
    .state-anim {
      animation: none !important;
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .scan-container {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 1000px;
    margin: 0 auto;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
  }

  .card-header h3 {
    margin: 0;
  }

  .scan-form {
    display: flex;
    gap: 12px;
  }

  input[type="text"] {
    flex: 1;
    padding: 12px 16px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--glass-border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 15px;
    font-family: inherit;
    transition: box-shadow 0.2s, border-color 0.2s;
  }

  input[type="text"]:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.3);
    border-color: var(--accent);
  }
  
  input[type="text"]:disabled {
    opacity: 0.6;
  }

  .btn { gap: 8px; }
  
  .spin {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-banner {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-top: 16px;
    padding: 16px;
    background: rgba(255, 59, 48, 0.1);
    color: var(--danger);
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .error-content {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }

  .rescan-btn {
    margin-top: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
  }

  .stat-value {
    font-size: 48px;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 8px;
    letter-spacing: -0.02em;
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .success-card {
    color: var(--success);
    background: rgba(52, 199, 89, 0.05);
    border-color: rgba(52, 199, 89, 0.2);
  }

  .findings-table {
    padding: 0;
    overflow: hidden;
  }

  .table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--glass-border);
  }
  
  .table-header h4 {
    margin: 0;
    font-size: 18px;
  }

  .table-wrapper {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
  }

  th, td {
    padding: 14px 24px;
    font-size: 14px;
    border-bottom: 1px solid var(--glass-border);
  }

  th {
    color: var(--text-secondary);
    font-weight: 500;
  }

  tr:last-child td {
    border-bottom: none;
  }

  .badge {
    display: inline-block;
    padding: 4px 10px;
    border-radius: 980px;
    font-weight: 600;
    font-size: 12px;
    letter-spacing: 0.02em;
  }

  .rule-col { font-weight: 500; }
  .path-col { color: var(--text-secondary); font-family: ui-monospace, monospace; font-size: 13px; }
  .context-col .code-block { white-space: nowrap; }
</style>
