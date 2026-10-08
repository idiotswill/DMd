<script lang="ts">
  import { onDestroy } from 'svelte';
  import { tableApi, type IntrinsicAttackOptions, type IntrinsicAttackRequest } from '../table-api';
  import type { TacticalAction } from '../tactical-api';
  let { request, disabled=false, onAction }: {
    request: IntrinsicAttackRequest; disabled?: boolean; onAction: (action: TacticalAction) => void;
  } = $props();
  let options = $state<IntrinsicAttackOptions | null>(null);
  let feature = $state(''); let target = $state('');
  let loading = $state(false); let error = $state('');
  let generation = 0; let alive = true;
  const identity = $derived(JSON.stringify(request));
  $effect(() => {
    // Any context or lock transition invalidates both late success and failure.
    identity; disabled;
    generation += 1;
    options = null; feature = ''; target = ''; loading = false; error = '';
  });
  onDestroy(() => { alive = false; generation += 1; });
  async function load() {
    if (disabled || loading) return;
    const ticket = ++generation; const observed = identity;
    const current = () => alive && !disabled && generation === ticket && identity === observed;
    loading = true; options = null; error = '';
    try {
      const answer = await tableApi.intrinsicAttackOptions(JSON.parse(observed) as IntrinsicAttackRequest);
      if (!current()) return;
      if (answer.version !== 1 || answer.actor !== request.actor || answer.revision !== request.revision) throw new Error('The creature attack choices changed. Refresh the table.');
      options = answer; feature = answer.features[0]?.feature_id ?? ''; target = '';
    } catch (reason) {
      if (current()) error = reason instanceof Error ? reason.message
        : typeof reason === 'object' && reason !== null && 'message' in reason ? String(reason.message)
        : typeof reason === 'string' ? reason : 'The creature attacks could not be loaded. Refresh the table.';
    } finally {
      if (current()) loading = false;
    }
  }
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (disabled || !options?.features.some(choice => choice.feature_id === feature)
      || !options.targets.some(choice => choice.actor === target)) return;
    onAction({ CreatureAttack: { target, feature_id: feature, weapon: null } });
  }
</script>
<section class="panel" data-tactical-focus="intrinsic-attack" tabindex="-1">
  <h2>Creature melee attack</h2>
  <button type="button" disabled={disabled || loading} onclick={load}>{loading ? 'Loading creature attacks…' : 'Choose creature attack'}</button>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if options}
    {#if options.features.length && options.targets.length}
      <form onsubmit={submit}><fieldset {disabled}><legend>Attack action</legend>
        <label>Creature attack<select bind:value={feature} required>{#each options.features as choice (choice.feature_id)}<option value={choice.feature_id}>{choice.label}</option>{/each}</select></label>
        <label>Creature attack target<select bind:value={target} required><option value="" disabled>Choose a located creature</option>{#each options.targets as choice (choice.actor)}<option value={choice.actor}>{choice.label}</option>{/each}</select></label>
        <button type="submit" disabled={!feature || !target}>Use creature attack</button>
      </fieldset></form>
    {:else}<p>No supported creature attack is available against a located target this turn.</p>{/if}
  {/if}
</section>
