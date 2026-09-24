<script lang="ts">
  import { onMount } from 'svelte';
  import { loadDesktopStatus, type DesktopStatus } from './bridge';
  import TableApp from './TableApp.svelte';

  let status = $state<DesktopStatus | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let errorMessage = $state('');

  async function connect() {
    loading = true;
    failed = false;
    try {
      status = await loadDesktopStatus();
    } catch (reason) {
      failed = true;
      status = null;
      errorMessage = reason && typeof reason === 'object' && 'message' in reason && typeof reason.message === 'string'
        ? reason.message : 'The window could not connect to DMd. Retry, or close and reopen the application.';
    } finally {
      loading = false;
    }
  }

  onMount(connect);
</script>

<a class="skip-link" href="#main">Skip to main content</a>
<header>
  <span class="brand-mark" aria-hidden="true">D</span>
  <div><strong>DMd</strong><span class="subtitle">Your tabletop, together</span></div>
  <span class="local-badge">Local campaign</span>
</header>
<main id="main" tabindex="-1" class:table-ready={status?.runtimeReady}>
  {#if status?.runtimeReady}
    <TableApp />
  {:else}
  <section class="welcome" aria-labelledby="welcome-title">
    <p class="eyebrow">Welcome to the table</p>
    <h1 id="welcome-title">A place for your next adventure.</h1>
    <p>Campaigns, characters and table conversation stay on this computer.</p>
  </section>
  <section class="status-card" aria-labelledby="connection-title" aria-busy={loading}>
    <h2 id="connection-title">{loading ? 'Opening DMd…' : failed ? 'Unable to connect' : 'Desktop status'}</h2>
    <div role="status" aria-live="polite">
      {#if loading}
        <p>Connecting to the local application.</p>
      {:else if failed}
        <p>{errorMessage}</p>
      {:else if status}
        <p>{status.message}</p>
        <p class="version">Version {status.version}</p>
      {/if}
    </div>
    {#if failed}
      <button onclick={connect} disabled={loading}>Retry connection</button>
    {/if}
  </section>
  {/if}
</main>
