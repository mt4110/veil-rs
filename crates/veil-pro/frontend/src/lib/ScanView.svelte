<script lang="ts">
  import { FolderSearch, AlertTriangle, ShieldCheck, Play, Loader2, RotateCcw, Download, Printer } from 'lucide-svelte';
  import type { ErrorEnvelope, PresetName, SafeFindingApiV1, ScanRequest, ScanResponse, SeverityName } from './api-contract';
  import type { ScanPreset, ScanState } from './ui-state';
  
  let currentState = $state<ScanState>('Idle');
  
  let scanPath = $state('');
  let scanPreset = $state<ScanPreset>('');
  let includeSuppressed = $state(false);
  let findings = $state<SafeFindingApiV1[]>([]);
  let scanStats = $state<{
    scanned: number;
    skipped: number;
    runId: string;
    effectiveFindings: number;
    suppressedFindings: number;
    totalFindings: number;
    coverageComplete: boolean;
    limitReached: boolean;
    limitReasons: string[];
  } | null>(null);
  let errorMsg = $state<string | null>(null);

  const sevWeights: Record<SeverityName, number> = { Critical: 4, High: 3, Medium: 2, Low: 1 };
  
  let sortedFindings = $derived(
    [...findings].sort((a, b) => {
      const statusA = a.baselineStatus === 'suppressed' ? 0 : 1;
      const statusB = b.baselineStatus === 'suppressed' ? 0 : 1;
      const weightA = sevWeights[a.severity];
      const weightB = sevWeights[b.severity];
      return statusB - statusA || weightB - weightA;
    })
  );

  function safeInt(v: unknown, fallback = 0): number {
    const n = Number.parseInt(String(v), 10);
    return Number.isFinite(n) ? n : fallback;
  }

  function incompleteScanMessage(data: ScanResponse): string {
    if (data.limitReasons.length > 0) {
      return 'Scan incomplete. Results are partial.';
    }
    if (data.limitReached) {
      return 'Scan limit reached. Results are partial.';
    }
    if (!data.coverageComplete) {
      return 'Scan coverage is incomplete. Results are partial.';
    }
    return 'Scan incomplete. Results are partial.';
  }

  async function readErrorMessage(res: Response, fallback: string): Promise<string> {
    const err = await res.json().catch(() => null) as Partial<ErrorEnvelope> & { error?: unknown; message?: unknown } | null;
    if (typeof err?.error === 'string') {
      return err.error;
    }
    if (err?.error && typeof err.error === 'object' && 'message' in err.error) {
      const message = err.error.message;
      if (typeof message === 'string') {
        return message;
      }
    }
    if (typeof err?.message === 'string') {
      return err.message;
    }
    return fallback;
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

      const requestBody: ScanRequest = { paths: [targetPath] };
      if (scanPreset !== '') {
        requestBody.preset = scanPreset;
      }
      if (includeSuppressed) {
        requestBody.includeSuppressed = true;
      }

      const res = await fetch('/api/scan', {
        method: 'POST',
        headers,
        body: JSON.stringify(requestBody)
      });

      if (res.status === 401) {
        currentState = 'ErrorAuth';
        errorMsg = 'Session expired or token invalid. Please login again.';
        return;
      }
      if (res.status === 400 || res.status === 422) {
        currentState = 'ErrorConfig';
        errorMsg = await readErrorMessage(res, 'Configuration error or invalid path.');
        return;
      }
      if (res.status === 413) {
        currentState = 'ErrorOOM';
        errorMsg = 'Target is too large. Resource limit exceeded.';
        return;
      }
      if (!res.ok) {
        errorMsg = await readErrorMessage(res, `HTTP ${res.status}`);
        currentState = 'ErrorUnknown';
        return;
      }

      const data = await res.json() as ScanResponse;
      findings = data.findings;
      scanStats = {
        scanned: data.scannedFiles,
        skipped: data.skippedFiles,
        runId: data.runId,
        effectiveFindings: data.effectiveFindings,
        suppressedFindings: data.suppressedFindings,
        totalFindings: data.totalFindings,
        coverageComplete: data.coverageComplete,
        limitReached: data.limitReached,
        limitReasons: data.limitReasons
      };
      
      if (data.status === 'incomplete' || data.limitReached || !data.coverageComplete) {
        errorMsg = incompleteScanMessage(data);
        currentState = 'Incomplete';
      } else if (data.effectiveFindings > 0) {
        currentState = 'Violation';
      } else {
        currentState = 'SuccessNoFindings';
      }
      
    } catch (err: unknown) {
      currentState = 'ErrorUnknown';
      errorMsg = err instanceof Error ? err.message : 'An unknown network error occurred';
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
        errorMsg = await readErrorMessage(
          res,
          `Failed to download evidence pack (HTTP ${res.status}).`
        );
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
    } catch (_err: unknown) {
      currentState = 'ErrorUnknown';
      errorMsg = 'Network error downloading evidence.';
    }
  }

  function printReport() {
    window.print();
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
      <select bind:value={scanPreset} aria-label="Preset" disabled={currentState === 'Running'}>
        <option value="">Default</option>
        <option value="standard-jp">standard-jp</option>
        <option value="fintech-jp">fintech-jp</option>
        <option value="gov-jp">gov-jp</option>
        <option value="si-vendor-jp">si-vendor-jp</option>
        <option value="logs-jp">logs-jp</option>
      </select>
      <label class="toggle-row">
        <input
          type="checkbox"
          bind:checked={includeSuppressed}
          disabled={currentState === 'Running'}
        />
        <span class="toggle-switch" aria-hidden="true"></span>
        <span class="toggle-label">Include Suppressed</span>
      </label>
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
          <span>{errorMsg || (currentState === 'Incomplete' ? 'Scan incomplete. Results are partial.' : 'An error occurred.')}</span>
          {#if currentState === 'Incomplete' && scanStats?.limitReasons.length}
            <div class="reason-chips" aria-label="Limit reasons">
              {#each scanStats.limitReasons as reason}
                <code>{reason}</code>
              {/each}
            </div>
          {/if}
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
      <div class="results-toolbar">
        <h4>Scan Results</h4>
        <div class="results-actions">
          <button class="export-link btn btn-secondary btn-sm" onclick={printReport} type="button" title="Print report">
            <Printer size={14} /> Print Report
          </button>
          <button class="export-link btn btn-secondary btn-sm" onclick={() => downloadEvidence(scanStats!.runId)} type="button">
            <Download size={14} /> Export Evidence Pack
          </button>
        </div>
      </div>

      <div class="stats-cards">
        <div class="stat-card glass-panel">
          <span class="stat-value">{safeInt(scanStats.scanned)}</span>
          <span class="stat-label">Files Scanned</span>
        </div>
        <div class="stat-card glass-panel">
          <span class="stat-value">{safeInt(scanStats.effectiveFindings)}</span>
          <span class="stat-label">Active Findings</span>
        </div>
        <div class="stat-card glass-panel" class:warning-card={!scanStats.coverageComplete || scanStats.limitReached}>
          <span class="stat-value text-value">{scanStats.coverageComplete && !scanStats.limitReached ? 'Full' : 'Partial'}</span>
          <span class="stat-label">Coverage</span>
        </div>
        {#if includeSuppressed || scanStats.suppressedFindings > 0}
          <div class="stat-card glass-panel">
            <span class="stat-value">{safeInt(scanStats.suppressedFindings)}</span>
            <span class="stat-label">Suppressed</span>
          </div>
        {/if}
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
          </div>
          
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Status</th>
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
                      <span class="badge status-{finding.baselineStatus === 'suppressed' ? 'suppressed' : 'active'}">
                        {finding.baselineStatus === 'suppressed' ? 'Suppressed' : 'Active'}
                      </span>
                    </td>
                    <td>
                      <!-- Strict CSP: Use CSS classes instead of inline style -->
                      <span class="badge sev-{String(finding.severity || '').toLowerCase()}">
                        {finding.severity}
                      </span>
                    </td>
                    <td class="rule-col">{finding.ruleId}</td>
                    <td class="path-col">{finding.path}:{safeInt(finding.lineNumber)}</td>
                    <td class="context-col">
                      <code class="code-block">{finding.maskedSnippet}</code>
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
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(160px, 220px) minmax(180px, auto) auto;
    gap: 12px;
  }

  input[type="text"],
  select {
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

  select {
    appearance: none;
    background-image: linear-gradient(45deg, transparent 50%, var(--text-secondary) 50%), linear-gradient(135deg, var(--text-secondary) 50%, transparent 50%);
    background-position: calc(100% - 18px) 50%, calc(100% - 13px) 50%;
    background-size: 5px 5px, 5px 5px;
    background-repeat: no-repeat;
    padding-right: 36px;
  }

  input[type="text"]:focus,
  select:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.3);
    border-color: var(--accent);
  }
  
  input[type="text"]:disabled,
  select:disabled {
    opacity: 0.6;
  }

  .toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    cursor: pointer;
  }

  .toggle-row input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  .toggle-switch {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: 999px;
    background: var(--glass-border);
    transition: background 0.2s, box-shadow 0.2s;
    flex: 0 0 auto;
  }

  .toggle-switch::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--bg-primary);
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
    transition: transform 0.2s;
  }

  .toggle-row input:checked + .toggle-switch {
    background: var(--accent);
  }

  .toggle-row input:checked + .toggle-switch::after {
    transform: translateX(16px);
  }

  .toggle-row input:focus-visible + .toggle-switch {
    box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.3);
  }

  .toggle-row input:disabled + .toggle-switch,
  .toggle-row input:disabled ~ .toggle-label {
    opacity: 0.6;
  }

  @media (max-width: 720px) {
    .scan-form {
      grid-template-columns: 1fr;
    }
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

  .reason-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .reason-chips code {
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid rgba(255, 59, 48, 0.2);
    font-size: 12px;
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

  .results-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }

  .results-toolbar h4 {
    margin: 0;
    font-size: 18px;
  }

  .export-link {
    flex: 0 0 auto;
  }

  .results-actions {
    display: inline-flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }

  @media (max-width: 720px) {
    .results-toolbar {
      align-items: stretch;
      flex-direction: column;
    }

    .results-actions {
      width: 100%;
    }

    .export-link {
      width: 100%;
    }
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

  .stat-value.text-value {
    font-size: 28px;
    letter-spacing: 0;
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

  .warning-card {
    color: var(--danger);
    background: rgba(255, 59, 48, 0.05);
    border-color: rgba(255, 59, 48, 0.2);
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

  .status-active {
    color: var(--success);
    background: rgba(52, 199, 89, 0.1);
  }

  .status-suppressed {
    color: var(--text-secondary);
    background: var(--bg-primary);
    border: 1px solid var(--glass-border);
  }

  .rule-col { font-weight: 500; }
  .path-col { color: var(--text-secondary); font-family: ui-monospace, monospace; font-size: 13px; }
  .context-col .code-block { white-space: nowrap; }
</style>
