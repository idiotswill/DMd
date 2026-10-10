<script lang="ts">
  import type { TableView } from '../table-api';
  let { view, host, disabled = false, onAward, onExcessAward }: {
    view: TableView; host: boolean; disabled?: boolean;
    onAward: (character_id: string, reason: string) => void;
    onExcessAward?: (character_id: string, reason: string) => void;
  } = $props();
  let character = $state('');
  let reason = $state('');
  let error = $state('');
  const candidates = $derived(view.characters.filter(c => c.player_id && c.profile && c.details));
  const selected = $derived(candidates.find(c => c.character_id === character));
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!selected || (selected.details?.heroic_inspiration && !onExcessAward) || !reason.trim() || new TextEncoder().encode(reason).length > 2000) {
      error = 'Select an eligible character and give a brief reason for the award.'; return;
    }
    error = '';
    if (selected.details?.heroic_inspiration && onExcessAward) onExcessAward(character, reason);
    else onAward(character, reason);
  }
</script>

{#if host && view.grapple?.version === 4 && view.active_session}
  <form onsubmit={submit}>
    <fieldset disabled={disabled || !!view.pending || !!view.roll || !!view.tactical?.continuation || !!view.tactical?.ready?.length}>
      <legend>Award Heroic Inspiration</legend>
      <p>As Host, record a GM award to a player character. A character can hold one Inspiration.{#if onExcessAward} If they already have it, their attending player chooses whether to give the extra Inspiration to another eligible character.{/if}</p>
      <label>Recipient<select required bind:value={character}><option value="">Select a character</option>{#each candidates as candidate}<option value={candidate.character_id} disabled={candidate.details?.heroic_inspiration && (!onExcessAward || !view.active_session?.participants?.some(p=>p.player_id===candidate.player_id&&p.character_id===candidate.character_id&&p.attendance==='Present'))}>{candidate.name}{candidate.details?.heroic_inspiration ? ' — already inspired' : ''}</option>{/each}</select></label>
      <label>Reason for the award<textarea required maxlength="2000" bind:value={reason}></textarea></label>
      {#if error}<p role="alert">{error}</p>{/if}
      <button type="submit">Award Heroic Inspiration</button>
    </fieldset>
  </form>
{/if}
