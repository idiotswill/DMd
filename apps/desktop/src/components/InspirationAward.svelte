<script lang="ts">
  import type { TableView } from '../table-api';
  let { view, host, disabled = false, onAward }: {
    view: TableView; host: boolean; disabled?: boolean;
    onAward: (character_id: string, reason: string) => void;
  } = $props();
  let character = $state('');
  let reason = $state('');
  let error = $state('');
  const candidates = $derived(view.characters.filter(c => c.player_id && c.profile && c.details));
  const selected = $derived(candidates.find(c => c.character_id === character));
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!selected || selected.details?.heroic_inspiration || !reason.trim() || new TextEncoder().encode(reason).length > 2000) {
      error = 'Select a character without Inspiration and give a brief reason for the award.'; return;
    }
    error = ''; onAward(character, reason);
  }
</script>

{#if host && view.grapple?.version === 4 && view.active_session}
  <form onsubmit={submit}>
    <fieldset disabled={disabled || !!view.pending || !!view.roll || !!view.tactical?.continuation || !!view.tactical?.ready?.length}>
      <legend>Award Heroic Inspiration</legend>
      <p>As Host, record a GM award to a player character. A character can hold one Inspiration; overflow transfer is not yet available.</p>
      <label>Recipient<select required bind:value={character}><option value="">Select a character</option>{#each candidates as candidate}<option value={candidate.character_id} disabled={candidate.details?.heroic_inspiration}>{candidate.name}{candidate.details?.heroic_inspiration ? ' — already inspired' : ''}</option>{/each}</select></label>
      <label>Reason for the award<textarea required maxlength="2000" bind:value={reason}></textarea></label>
      {#if error}<p role="alert">{error}</p>{/if}
      <button type="submit">Award Heroic Inspiration</button>
    </fieldset>
  </form>
{/if}
