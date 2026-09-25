<script lang="ts">
  import type { TacticalAction } from '../tactical-api';
  let { disabled = false, onAction }: { disabled?: boolean; onAction: (action:TacticalAction)=>void } = $props();
  let continueCadence = $state(false);
  let ruling = $state('');
  function conclude(event: SubmitEvent) {
    event.preventDefault();
    if (!continueCadence || !ruling.trim()) return;
    onAction({ConcludeHostilities:{cadence:'ContinueExistingOrder',ruling:ruling.trim()}});
  }
</script>
<form onsubmit={conclude}>
  <fieldset {disabled}><legend>Conclude hostilities</legend>
    <p>The encounter map and turn order remain in use for ongoing saves, durations and readied actions. This does not heal anyone, restore resources or advance time.</p>
    <label class="check"><input type="checkbox" required bind:checked={continueCadence} />As host, keep the existing turn order for aftermath timing.</label>
    <label>Private host timing ruling<textarea required maxlength="2000" rows="2" bind:value={ruling}></textarea></label>
    <p class="muted">This is a GM timing choice. Players still decide their actions and report their own dice. You can end the session once pending work is settled, then resume the same cadence.</p>
    <button type="submit" disabled={!continueCadence || !ruling.trim()}>Conclude hostilities and retain timing</button>
  </fieldset>
</form>
