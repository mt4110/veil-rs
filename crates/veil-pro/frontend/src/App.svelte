<script lang="ts">
  import { onMount } from 'svelte';
  import Login from './lib/Login.svelte';
  import Dashboard from './lib/Dashboard.svelte';

  let isAuthenticated = $state(false);
  let authType = $state<string | null>(null);
  let userEmail = $state<string | null>(null);
  let userName = $state<string | null>(null);
  let initialized = $state(false);

  onMount(async () => {
    // 1. Check URL Fragment for CLI token injection
    const hash = window.location.hash;
    if (hash.startsWith('#token=')) {
      const tokenStr = hash.substring(7);
      sessionStorage.setItem('veil_token', tokenStr);
      // Clean URL for B2B security (no tokens in copy/paste)
      window.history.replaceState(null, '', window.location.pathname);
    }

    // 2. Validate session/token via /api/me
    try {
      const headers: Record<string, string> = {};
      const storedToken = sessionStorage.getItem('veil_token');
      if (storedToken) {
        headers['Authorization'] = `Bearer ${storedToken}`;
      }

      const res = await fetch('/api/me', { headers });
      if (res.ok) {
        const data = await res.json();
        isAuthenticated = true;
        authType = data.type;
        userEmail = data.email || null;
        userName = data.name || null;
      } else {
        // Clear stale local token if server rejected it
        if (storedToken && res.status === 401) {
          sessionStorage.removeItem('veil_token');
        }
      }
    } catch (e) {
        console.error("Auth check failed:", e);
    }
    initialized = true;
  });
</script>

{#if !initialized}
  <div class="loader-container">
    <div class="spinner"></div>
    <p>Authenticating...</p>
  </div>
{:else if isAuthenticated}
  <Dashboard {authType} {userName} {userEmail} />
{:else}
  <Login />
{/if}

<style>
  .loader-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
  }
  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--glass-border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 16px;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
