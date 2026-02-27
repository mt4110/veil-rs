<script lang="ts">
  import { Shield, LayoutDashboard, ScanSearch, FileCheck2, Settings, UserCircle, LogOut } from 'lucide-svelte';
  import ScanView from './ScanView.svelte';

  // v1.0.0 strict local-first architecture (No SSO, Local Token Auth)
  let currentView = $state('scan');
</script>

<div class="dashboard-layout">
  <aside class="sidebar glass-panel">
    <div class="brand">
      <Shield size={28} color="var(--accent)" />
      <span>Veil Pro</span>
    </div>

    <nav class="nav-links">
      <div class="nav-section">
        <span class="section-title">Daily Triage</span>
        <button class:active={currentView === 'scan'} onclick={() => currentView = 'scan'}>
          <ScanSearch size={18} /> New Scan
        </button>
      </div>

      <div class="nav-section">
        <span class="section-title">Governance</span>
        <button class:active={currentView === 'policy'} onclick={() => currentView = 'policy'}>
          <FileCheck2 size={18} /> Organization Policy
        </button>
      </div>

      <div class="nav-section">
        <span class="section-title">System</span>
        <button class:active={currentView === 'settings'} onclick={() => currentView = 'settings'}>
          <Settings size={18} /> Configuration
        </button>
      </div>
    </nav>

    <div class="user-profile">
      <UserCircle size={32} color="var(--text-secondary)" />
      <div class="user-info">
        <strong>Local Admin</strong>
        <span>CLI Session</span>
      </div>
    </div>
  </aside>

  <main class="content-area">
    <header class="topbar glass-panel">
      <!-- svelte-ignore a11y_missing_content -->
      <h2>
        {#if currentView === 'scan'}
          Secret Triage
        {:else if currentView === 'policy'}
          Organization Policy (VEIL_ORG_CONFIG)
        {:else}
          Configuration
        {/if}
      </h2>
      <div class="actions">
        <button class="btn btn-secondary btn-sm" onclick={() => {
            sessionStorage.removeItem('veil_token');
            window.location.href = '/';
        }}>
          <LogOut size={16} style="margin-right: 6px;" /> End Session
        </button>
      </div>
    </header>

    <div class="scroll-content">
      {#if currentView === 'scan'}
        <ScanView />
      {:else if currentView === 'policy'}
        <div class="glass-panel overview-hero state-anim">
           <h3>Governance Policy</h3>
           <p>Active policy applies to all scans in this session.</p>
           <!-- Future PolicyPanel integration goes here -->
        </div>
      {:else}
         <div class="glass-panel overview-hero state-anim">
           <h3>Configuration</h3>
           <p>Manage Baseline overrides and local thresholds.</p>
           <!-- Future Config/Baseline Panel integration goes here -->
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  /* CSP Note: No inline styles. Native CSS transitions only. */
  .state-anim {
    animation: fadeIn 0.15s ease-out forwards;
  }
  
  @media (prefers-reduced-motion: reduce) {
    .state-anim {
      animation: none !important;
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(2px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .dashboard-layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .sidebar {
    border-radius: 0;
    border-top: none;
    border-bottom: none;
    border-left: none;
    display: flex;
    flex-direction: column;
    padding: 24px 16px;
    z-index: 10;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 32px;
    padding: 0 12px;
  }

  .nav-links {
    display: flex;
    flex-direction: column;
    gap: 24px;
    flex: 1;
  }

  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .section-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
    padding: 0 14px;
    margin-bottom: 2px;
    font-weight: 600;
  }

  .nav-links button {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
  }

  .nav-links button:hover {
    background: rgba(0,0,0,0.05);
  }

  @media (prefers-color-scheme: dark) {
    .nav-links button:hover {
      background: rgba(255,255,255,0.1);
    }
  }

  .nav-links button.active {
    background: var(--accent);
    color: white;
  }

  .user-profile {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    border: 1px solid var(--glass-border);
  }

  .user-info {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .user-info strong {
    font-size: 14px;
    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
  }

  .user-info span {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
  }

  .content-area {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .topbar {
    border-radius: 0;
    border-top: none;
    border-left: none;
    border-right: none;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 32px;
    z-index: 5;
  }

  .topbar h2 {
    margin: 0;
    font-size: 20px;
  }

  .btn-sm {
    padding: 6px 14px;
    font-size: 13px;
  }

  .scroll-content {
    flex: 1;
    overflow-y: auto;
    padding: 32px;
  }

  .overview-hero {
    margin-bottom: 24px;
  }
</style>
