<script lang="ts">
  import { untrack } from 'svelte';
  import type { TableContract } from '../table-api';
  let { value, disabled = false, mechanicsLocked = false, submitLabel = 'Save table agreement', onSave }: { value: TableContract; disabled?: boolean; mechanicsLocked?: boolean; submitLabel?: string; onSave: (value: TableContract) => void } = $props();
  let draft = $state(untrack(() => structuredClone($state.snapshot(value))));
  const fields = [
    ['tone', 'Tone'], ['humor', 'Seriousness and comedy'], ['tactical_preference', 'Tactical combat preference'],
    ['exploration_preference', 'Exploration preference'], ['social_preference', 'Social roleplay preference'],
    ['lethality', 'Expected lethality'], ['advancement', 'Advancement agreement'], ['house_rule_notes', 'Additional house-rule discussion'],
    ['pvp_policy', 'Player-versus-player policy'], ['theft_and_secrets', 'PC theft and secret actions'],
    ['retcon_policy', 'Corrections and retcons'], ['content_boundaries', 'Content boundaries'],
    ['explanation_depth', 'Rules explanation depth'], ['experience', 'Player experience'], ['absent_player_policy', 'Absent players'],
  ] as const;
</script>

<form onsubmit={(event) => { event.preventDefault(); onSave(structuredClone($state.snapshot(draft))); }}>
  <fieldset {disabled}>
    <legend>Session Zero — your table agreement</legend>
    <p>Agree on these preferences together. They are saved with this campaign.</p>
    <div class="form-grid">
      {#each fields as [key, title]}
        <label>{title}<textarea rows={key === 'content_boundaries' ? 3 : 2} required maxlength="4000" bind:value={draft[key]}></textarea></label>
      {/each}
    </div>
    <p><strong>Rules:</strong> SRD {draft.ruleset.version}. <strong>Supported creation:</strong> Human, Fighter, Soldier. Additional content choices are not available yet.</p>
    <p><strong>Optional rules:</strong> none selected; additional optional rules are not available yet. Agreement notes do not add automated mechanics.</p>
    <label class="check"><input type="checkbox" bind:checked={draft.house_rules.ability_test_natural_extremes} disabled={mechanicsLocked} />House rule: natural 1 and 20 automatically fail and succeed on ability tests</label>
    {#if mechanicsLocked}<p class="muted">Mechanical house rules are locked after character creation. Other agreements can be updated between sessions.</p>{/if}
    <button type="submit">{submitLabel}</button>
  </fieldset>
</form>
