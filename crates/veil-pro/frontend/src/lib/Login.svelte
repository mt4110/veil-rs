<script lang="ts">
</script>

<div class="login-wrapper state-anim">
  <div class="glass-panel login-box state-anim-delayed">
    <div class="logo-area">
      <div class="shield-icon">🛡️</div>
      <h1>Veil Pro</h1>
      <p>Secure B2B Governance Dashboard</p>
    </div>

    <!-- B2B Trust Panel -->
    <div class="trust-panel">
      <div class="trust-item">
        <span class="trust-icon">🔒</span>
        <div class="trust-text">
          <strong>Local-First Architecture</strong>
          <span>コードや結果は外部送信されません</span>
        </div>
      </div>
      <div class="trust-item">
        <span class="trust-icon">⚡</span>
        <div class="trust-text">
          <strong>Zero Telemetry</strong>
          <span>完全にローカル環境で完結します</span>
        </div>
      </div>
    </div>

    <div class="cli-info">
      <p>CLIから起動するか、発行されたローカルリンクを使用してください:</p>
      <code>veil scan . --ui</code>
    </div>
  </div>
</div>

<style>
  /* CSP Note: No inline styles or Svelte transitions */
  .state-anim {
    animation: fadeIn 0.4s ease-out forwards;
  }
  
  .state-anim-delayed {
    opacity: 0;
    animation: flyUp 0.6s ease-out 0.1s forwards;
  }

  @media (prefers-reduced-motion: reduce) {
    .state-anim, .state-anim-delayed {
      animation: none !important;
      opacity: 1;
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes flyUp {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .login-wrapper {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at 50% 0%, rgba(0,113,227,0.05) 0%, transparent 60%), var(--bg-primary);
  }

  .login-box {
    width: 100%;
    max-width: 420px;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 24px;
    border-radius: var(--radius-lg);
  }

  .logo-area {
    margin-bottom: 8px;
  }

  .shield-icon {
    font-size: 56px;
    margin-bottom: 12px;
    filter: drop-shadow(0 4px 8px rgba(0,0,0,0.1));
  }

  .logo-area h1 {
    margin-bottom: 4px;
  }

  .logo-area p {
    font-size: 14px;
    margin-bottom: 0px;
    color: var(--text-secondary);
  }

  .trust-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: rgba(0, 0, 0, 0.02);
    padding: 20px;
    border-radius: var(--radius-md);
    border: 1px solid var(--glass-border);
    text-align: left;
  }

  @media (prefers-color-scheme: dark) {
    .trust-panel {
      background: rgba(255, 255, 255, 0.03);
    }
  }

  .trust-item {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .trust-icon {
    font-size: 20px;
  }

  .trust-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .trust-text strong {
    font-size: 14px;
    font-weight: 600;
  }

  .trust-text span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .cli-info {
    margin-top: 8px;
  }

  .cli-info p {
    font-size: 13px;
    margin-bottom: 12px;
    color: var(--text-secondary);
  }

  .cli-info code {
    background: var(--bg-secondary);
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 14px;
    border: 1px solid var(--glass-border);
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-weight: 500;
  }
</style>
