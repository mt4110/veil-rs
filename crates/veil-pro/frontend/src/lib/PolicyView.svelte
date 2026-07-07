<script lang="ts">
  import { onMount } from 'svelte';
  import { AlertTriangle, CheckCircle2, FileCheck2, RefreshCcw } from 'lucide-svelte';
  import type { ConfigLayerName, ErrorEnvelope, PolicyResponse } from './api-contract';
  import type { PolicyViewState } from './ui-state';

  let viewState = $state<PolicyViewState>({ kind: 'Loading' });

  const layerLabels: Record<ConfigLayerName, string> = {
    builtin: 'Builtin',
    preset: 'Preset',
    org: 'Org',
    repo: 'Repo',
    cli: 'CLI'
  };

  onMount(() => {
    loadPolicy();
  });

  function authHeaders(): Record<string, string> {
    const headers: Record<string, string> = {};
    const token = sessionStorage.getItem('veil_token');
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    return headers;
  }

  async function readErrorMessage(res: Response, fallback: string): Promise<string> {
    const err = await res.json().catch(() => null) as Partial<ErrorEnvelope> & { error?: unknown; message?: unknown } | null;
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

  async function loadPolicy() {
    viewState = { kind: 'Loading' };

    try {
      const res = await fetch('/api/policy', { headers: authHeaders() });
      if (res.status === 401) {
        viewState = {
          kind: 'ErrorAuth',
          message: 'Session expired. Please login again.'
        };
        return;
      }
      if (!res.ok) {
        viewState = {
          kind: 'ErrorUnknown',
          message: await readErrorMessage(res, `Failed to load policy (HTTP ${res.status}).`)
        };
        return;
      }
      viewState = {
        kind: 'Ready',
        policy: await res.json() as PolicyResponse
      };
    } catch (err: unknown) {
      viewState = {
        kind: 'ErrorUnknown',
        message: err instanceof Error ? err.message : 'Network error loading policy.'
      };
    }
  }

  function layerLabel(layer: string): string {
    return layerLabels[layer as ConfigLayerName] ?? layer;
  }
</script>

{#if viewState.kind === 'Loading'}
  <div class="glass-panel policy-state state-anim">
    <span class="spin">
      <RefreshCcw size={18} />
    </span>
    <span>Loading policy...</span>
  </div>
{:else if viewState.kind === 'ErrorAuth' || viewState.kind === 'ErrorUnknown'}
  <div class="glass-panel policy-state error state-anim">
    <AlertTriangle size={18} />
    <span>{viewState.message}</span>
    <button class="btn btn-sm btn-secondary" onclick={loadPolicy} type="button">
      <RefreshCcw size={14} /> Retry
    </button>
  </div>
{:else if viewState.kind === 'Ready'}
  <div class="policy-container state-anim">
    <div class="policy-summary">
      <div class="metric glass-panel">
        <span class="metric-value">{viewState.policy.effectiveRulesCount}</span>
        <span class="metric-label">Rules</span>
      </div>
      <div class="metric glass-panel">
        <span class="metric-value text-value">{viewState.policy.preset || 'Default'}</span>
        <span class="metric-label">Preset</span>
      </div>
      <div class="metric glass-panel" class:active={viewState.policy.hasOrgConfig}>
        <span class="metric-value text-value">{viewState.policy.hasOrgConfig ? 'Loaded' : 'None'}</span>
        <span class="metric-label">Org Config</span>
      </div>
      <div class="metric glass-panel">
        <span class="metric-value text-value">{viewState.policy.repoConfigPath ? 'Loaded' : 'None'}</span>
        <span class="metric-label">Repo Config</span>
      </div>
    </div>

    <section class="policy-section glass-panel">
      <div class="section-heading">
        <FileCheck2 size={20} color="var(--accent)" />
        <h3>Config Layers</h3>
      </div>
      <div class="layer-list">
        {#each viewState.policy.layers as layer}
          <div class="layer-row" class:loaded={layer.loaded}>
            <div class="layer-status">
              {#if layer.loaded}
                <CheckCircle2 size={16} />
              {:else}
                <span class="empty-status"></span>
              {/if}
            </div>
            <div class="layer-main">
              <strong>{layerLabel(layer.name)}</strong>
              <span>{layer.path || 'Not set'}</span>
            </div>
            <span class="warning-count" class:warn={layer.warnings.length > 0}>
              {layer.warnings.length} warnings
            </span>
          </div>
        {/each}
      </div>
    </section>

    <section class="policy-section glass-panel">
      <div class="section-heading">
        <AlertTriangle size={20} color={viewState.policy.conflicts.length > 0 ? 'var(--warning)' : 'var(--text-secondary)'} />
        <h3>Config Conflicts</h3>
      </div>
      {#if viewState.policy.conflicts.length > 0}
        <div class="conflict-list">
          {#each viewState.policy.conflicts as conflict}
            <div class="conflict-row">
              <strong>{conflict.key}</strong>
              <span>{layerLabel(conflict.winner)} wins over {conflict.shadowed.map(layerLabel).join(', ')}</span>
              <p>{conflict.explanation}</p>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty-copy">No config conflicts.</p>
      {/if}
    </section>
  </div>
{/if}

<style>
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

  .policy-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 1000px;
    margin: 0 auto;
  }

  .policy-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
  }

  .metric {
    align-items: center;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 132px;
    text-align: center;
  }

  .metric.active {
    border-color: rgba(52, 199, 89, 0.24);
  }

  .metric-value {
    font-size: 40px;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 8px;
  }

  .metric-value.text-value {
    font-size: 22px;
    line-height: 1.2;
    overflow-wrap: anywhere;
  }

  .metric-label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .policy-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-heading {
    align-items: center;
    display: flex;
    gap: 10px;
  }

  .section-heading h3 {
    font-size: 18px;
    margin: 0;
  }

  .layer-list,
  .conflict-list {
    display: flex;
    flex-direction: column;
  }

  .layer-row,
  .conflict-row {
    border-top: 1px solid var(--glass-border);
    display: grid;
    gap: 12px;
    padding: 14px 0;
  }

  .layer-row {
    align-items: center;
    grid-template-columns: 24px minmax(0, 1fr) auto;
  }

  .layer-row.loaded .layer-status {
    color: var(--success);
  }

  .layer-status {
    align-items: center;
    display: flex;
    justify-content: center;
  }

  .empty-status {
    background: var(--glass-border);
    border-radius: 50%;
    display: inline-block;
    height: 8px;
    width: 8px;
  }

  .layer-main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .layer-main strong,
  .conflict-row strong {
    font-size: 14px;
  }

  .layer-main span,
  .conflict-row span,
  .empty-copy {
    color: var(--text-secondary);
    font-size: 13px;
    overflow-wrap: anywhere;
  }

  .warning-count {
    border: 1px solid var(--glass-border);
    border-radius: 999px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    white-space: nowrap;
  }

  .warning-count.warn {
    color: var(--warning);
  }

  .conflict-row {
    grid-template-columns: minmax(120px, 240px) minmax(0, 1fr);
  }

  .conflict-row p {
    grid-column: 1 / -1;
    margin: 0;
  }

  .policy-state {
    align-items: center;
    display: flex;
    gap: 10px;
    margin: 0 auto;
    max-width: 1000px;
  }

  .policy-state.error {
    color: var(--danger);
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 720px) {
    .layer-row,
    .conflict-row {
      grid-template-columns: 1fr;
    }

    .layer-status {
      display: none;
    }
  }
</style>
