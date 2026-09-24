<script lang="ts">
  import { rawDice, signed, type RollRequest } from '../table-api';
  let { request, disabled = false, onSubmit }: { request: RollRequest; disabled?: boolean; onSubmit: (faces: number[]) => void } = $props();
  const sides = $derived(rawDice(request));
  let faces = $state<number[]>([]);
  let error = $state('');
  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (faces.length !== sides.length || sides.some((side, i) => !Number.isInteger(faces[i]) || faces[i] < 1 || faces[i] > side)) { error = 'Enter one valid raw face for every requested die.'; return; }
    error = ''; onSubmit([...faces]);
  }
</script>
<form onsubmit={submit}><fieldset {disabled}>
  <legend>Report physical dice</legend><p>{request.reason}</p>
  <p>{request.mode === 'Normal' ? 'Normal roll' : request.mode} · rules modifier {signed(request.modifier)}. Enter the faces exactly as rolled; do not add the modifier.</p>
  <div class="form-grid">{#each sides as side, i}<label>Die {i+1} · d{side}<input required type="number" min="1" max={side} step="1" bind:value={faces[i]} /></label>{/each}</div>
  {#if error}<p role="alert" class="error">{error}</p>{/if}<button type="submit">Report these faces</button>
</fieldset></form>
